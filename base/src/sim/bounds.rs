use std::rc::Rc;
use std::cell::RefCell;

/// Shared handle — both the constraint (in the solver) and the creator can
/// hold a clone and mutate or read the bounds independently each tick.
/// `Rc<RefCell>` is correct here: simulations run single-threaded.
pub type SharedBounds<B> = Rc<RefCell<B>>;

/// Convenience constructor — wraps any bounds value into a shared handle.
pub fn shared<B>(b: B) -> SharedBounds<B> {
    Rc::new(RefCell::new(b))
}

// ---------------------------------------------------------------------------
// Bounds trait
// ---------------------------------------------------------------------------

/// Spatial region a simulation lives inside.
/// Implemented by [`AabbBounds`], [`CircleBounds`], and [`SphereBounds`].
///
/// Used by [`BoundsConstraint`] (solver side) and [`VolumeCreator`] (creator side).
/// Both hold a [`SharedBounds<B>`] clone so either can resize the region at runtime.
pub trait Bounds<const N: usize> {
    /// Total interior volume (area in 2-D).
    fn volume(&self) -> f64;

    /// Returns `true` if `pos` is strictly inside the bounds.
    fn contains(&self, pos: &[f64; N]) -> bool;

    /// Geometric centre — used as the spawn point for new particles.
    fn center(&self) -> [f64; N];

    /// If `pos` is outside the bounds: clamp it back to the surface and
    /// reflect the normal component of `vel`, scaled by `restitution`.
    /// No-op if `pos` is already inside.
    /// Used by [`BoundsConstraint`] (AoS).
    fn clamp_reflect(&self, pos: &mut [f64; N], vel: &mut [f64; N], restitution: f64);

    /// Column-slice form for SoA storage.
    /// Layout: blocked — component `c` of particle `i` is `slice[i + c * len]`.
    /// Called once per `post()` over the full position and velocity columns.
    /// The inner loop per particle is identical to `clamp_reflect` —
    /// the compiler can auto-vectorize across `len` particles.
    /// Used by [`SoaBoundsConstraint`] (SoA).
    fn clamp_reflect_columns(&self, pos: &mut [f64], vel: &mut [f64], len: usize, restitution: f64);
}

// ---------------------------------------------------------------------------
// AabbBounds<N> — axis-aligned bounding box, any dimension.
//   N = 2 → rectangle   (use type alias `Rect`)
//   N = 3 → box         (use type alias `Aabb`)
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box.
/// - 2-D: use the [`Rect`] alias.
/// - 3-D: use the [`Aabb`] alias.
pub struct AabbBounds<const N: usize> {
    pub min: [f64; N],
    pub max: [f64; N],
}

impl<const N: usize> AabbBounds<N> {
    pub fn new(min: [f64; N], max: [f64; N]) -> Self { Self { min, max } }
}

/// 2-D axis-aligned rectangle.
pub type Rect = AabbBounds<2>;

/// 3-D axis-aligned box.
pub type Aabb = AabbBounds<3>;

impl<const N: usize> Bounds<N> for AabbBounds<N> {
    fn volume(&self) -> f64 {
        (0..N).fold(1.0, |v, i| v * (self.max[i] - self.min[i]))
    }

    fn contains(&self, pos: &[f64; N]) -> bool {
        (0..N).all(|i| pos[i] >= self.min[i] && pos[i] <= self.max[i])
    }

    fn center(&self) -> [f64; N] {
        let mut c = [0.0; N];
        for i in 0..N { c[i] = (self.min[i] + self.max[i]) * 0.5; }
        c
    }

    #[inline(always)]
    fn clamp_reflect(&self, pos: &mut [f64; N], vel: &mut [f64; N], restitution: f64) {
        for i in 0..N {
            if pos[i] < self.min[i] {
                pos[i] = self.min[i];
                vel[i] =  vel[i].abs() * restitution;
            } else if pos[i] > self.max[i] {
                pos[i] = self.max[i];
                vel[i] = -vel[i].abs() * restitution;
            }
        }
    }

    #[inline(always)]
    fn clamp_reflect_columns(&self, pos: &mut [f64], vel: &mut [f64], len: usize, restitution: f64) {
        // Each component c occupies pos[c*len .. (c+1)*len].
        // Iterate components in the outer loop so the inner loop over particles
        // is a contiguous stride-1 pass — auto-vectorizable.
        for c in 0..N {
            let lo  = self.min[c];
            let hi  = self.max[c];
            let col = c * len;
            for i in 0..len {
                let p = &mut pos[col + i];
                let v = &mut vel[col + i];
                if *p < lo {
                    *p =  lo;
                    *v =  v.abs() * restitution;
                } else if *p > hi {
                    *p =  hi;
                    *v = -v.abs() * restitution;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CircleBounds — 2-D disc.
// ---------------------------------------------------------------------------

/// Circular boundary in 2-D.
/// Particles outside the radius are projected back onto the surface and
/// their velocity is reflected through the outward normal.
pub struct CircleBounds {
    pub center: [f64; 2],
    pub radius: f64,
}

impl CircleBounds {
    pub fn new(center: [f64; 2], radius: f64) -> Self { Self { center, radius } }
}

impl Bounds<2> for CircleBounds {
    fn volume(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }

    fn contains(&self, pos: &[f64; 2]) -> bool {
        let dx = pos[0] - self.center[0];
        let dy = pos[1] - self.center[1];
        dx * dx + dy * dy <= self.radius * self.radius
    }

    fn center(&self) -> [f64; 2] { self.center }

    #[inline(always)]
    fn clamp_reflect(&self, pos: &mut [f64; 2], vel: &mut [f64; 2], restitution: f64) {
        let dx      = pos[0] - self.center[0];
        let dy      = pos[1] - self.center[1];
        let dist_sq = dx * dx + dy * dy;
        if dist_sq > self.radius * self.radius {
            let dist = dist_sq.sqrt();
            let nx   = dx / dist;
            let ny   = dy / dist;
            pos[0] = self.center[0] + nx * self.radius;
            pos[1] = self.center[1] + ny * self.radius;
            let vdotn = vel[0] * nx + vel[1] * ny;
            vel[0] = (vel[0] - 2.0 * vdotn * nx) * restitution;
            vel[1] = (vel[1] - 2.0 * vdotn * ny) * restitution;
        }
    }

    /// Column layout: pos[0..len] = x values, pos[len..2*len] = y values.
    #[inline(always)]
    fn clamp_reflect_columns(&self, pos: &mut [f64], vel: &mut [f64], len: usize, restitution: f64) {
        let r2  = self.radius * self.radius;
        let cx  = self.center[0];
        let cy  = self.center[1];
        for i in 0..len {
            let dx      = pos[i]       - cx;
            let dy      = pos[len + i] - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > r2 {
                let dist  = dist_sq.sqrt();
                let nx    = dx / dist;
                let ny    = dy / dist;
                pos[i]       = cx + nx * self.radius;
                pos[len + i] = cy + ny * self.radius;
                let vdotn    = vel[i] * nx + vel[len + i] * ny;
                vel[i]       = (vel[i]       - 2.0 * vdotn * nx) * restitution;
                vel[len + i] = (vel[len + i] - 2.0 * vdotn * ny) * restitution;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SphereBounds — 3-D ball.
// ---------------------------------------------------------------------------

/// Spherical boundary in 3-D.
/// Particles outside the radius are projected back onto the surface and
/// their velocity is reflected through the outward normal.
pub struct SphereBounds {
    pub center: [f64; 3],
    pub radius: f64,
}

impl SphereBounds {
    pub fn new(center: [f64; 3], radius: f64) -> Self { Self { center, radius } }
}

impl Bounds<3> for SphereBounds {
    fn volume(&self) -> f64 {
        (4.0 / 3.0) * std::f64::consts::PI * self.radius.powi(3)
    }

    fn contains(&self, pos: &[f64; 3]) -> bool {
        let d: f64 = (0..3).map(|i| (pos[i] - self.center[i]).powi(2)).sum();
        d <= self.radius * self.radius
    }

    fn center(&self) -> [f64; 3] { self.center }

    #[inline(always)]
    fn clamp_reflect(&self, pos: &mut [f64; 3], vel: &mut [f64; 3], restitution: f64) {
        let d: [f64; 3] = std::array::from_fn(|i| pos[i] - self.center[i]);
        let dist_sq: f64 = d.iter().map(|x| x * x).sum();
        if dist_sq > self.radius * self.radius {
            let dist  = dist_sq.sqrt();
            let n: [f64; 3] = std::array::from_fn(|i| d[i] / dist);
            for i in 0..3 { pos[i] = self.center[i] + n[i] * self.radius; }
            let vdotn: f64 = (0..3).map(|i| vel[i] * n[i]).sum();
            for i in 0..3 { vel[i] = (vel[i] - 2.0 * vdotn * n[i]) * restitution; }
        }
    }

    /// Column layout: pos[0..len]=x, pos[len..2*len]=y, pos[2*len..3*len]=z.
    #[inline(always)]
    fn clamp_reflect_columns(&self, pos: &mut [f64], vel: &mut [f64], len: usize, restitution: f64) {
        let r2 = self.radius * self.radius;
        for i in 0..len {
            let dx      = pos[i]           - self.center[0];
            let dy      = pos[len + i]     - self.center[1];
            let dz      = pos[2 * len + i] - self.center[2];
            let dist_sq = dx*dx + dy*dy + dz*dz;
            if dist_sq > r2 {
                let dist  = dist_sq.sqrt();
                let nx = dx / dist;  let ny = dy / dist;  let nz = dz / dist;
                pos[i]           = self.center[0] + nx * self.radius;
                pos[len + i]     = self.center[1] + ny * self.radius;
                pos[2 * len + i] = self.center[2] + nz * self.radius;
                let vdotn = vel[i] * nx + vel[len + i] * ny + vel[2 * len + i] * nz;
                vel[i]           = (vel[i]           - 2.0 * vdotn * nx) * restitution;
                vel[len + i]     = (vel[len + i]     - 2.0 * vdotn * ny) * restitution;
                vel[2 * len + i] = (vel[2 * len + i] - 2.0 * vdotn * nz) * restitution;
            }
        }
    }
}
