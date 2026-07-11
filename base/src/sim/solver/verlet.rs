use crate::math::{Bounds, DVec2};

/// `acc -= coeff * (pos - pos_old) / dt` — viscous drag for one component.
/// Velocity is approximated from the displacement; no stored velocity needed.
pub struct LinearDrag {
    pub coeff: f64,
}
impl LinearDrag {
    pub fn new(coeff: f64) -> Self { Self { coeff } }
    #[inline(always)]
    pub fn apply(&self, pos: f64, pos_old: f64, dt: f64, acc: &mut f64) {
        let vel = (pos - pos_old) / dt;
        *acc -= self.coeff * vel;
    }
}

/// Scale the implicit velocity by damping the displacement.
/// `pos_old = pos - (pos - pos_old) * factor` — equivalent to `vel *= factor`.
pub struct Damping {
    pub factor: f64,
}
impl Damping {
    pub fn new(factor: f64) -> Self { Self { factor } }
    #[inline(always)]
    pub fn apply(&self, pos: f64, pos_old: &mut f64) {
        *pos_old = pos - (pos - *pos_old) * self.factor;
    }
}

/// Clamp a particle inside a ball (circle in 2D, sphere in 3D) centred at `center` with `radius`.
/// Reconstructs `pos_old` so the next Verlet step produces the reflected velocity.
/// `restitution` ∈ `[0.0, 1.0]` — `1.0` = perfectly elastic, `0.0` = fully inelastic.
pub struct BallConstraint<const N: usize> {
    pub center:      [f64; N],
    pub radius:      f64,
    pub restitution: f64,
}
impl<const N: usize> BallConstraint<N> {
    pub fn new(center: [f64; N], radius: f64, restitution: f64) -> Self {
        Self { center, radius, restitution }
    }
    #[inline(always)]
    pub fn apply(&self, pos: &mut [f64; N], pos_old: &mut [f64; N]) {
        // Vector from center to particle.
        let mut delta = [0.0f64; N];
        let mut dist2 = 0.0f64;
        for i in 0..N {
            delta[i] = pos[i] - self.center[i];
            dist2 += delta[i] * delta[i];
        }
        let r2 = self.radius * self.radius;
        if dist2 > r2 {
            let dist = dist2.sqrt();
            let inv  = 1.0 / dist;
            // Surface normal pointing inward.
            let mut n = [0.0f64; N];
            for i in 0..N { n[i] = delta[i] * inv; }
            // Implicit velocity: vel = pos - pos_old
            let mut vel = [0.0f64; N];
            for i in 0..N { vel[i] = pos[i] - pos_old[i]; }
            // Clamp position to surface.
            for i in 0..N { pos[i] = self.center[i] + n[i] * self.radius; }
            // Reflect implicit velocity: v' = v - (1 + e) * dot(v, n) * n
            let dot: f64 = (0..N).map(|i| vel[i] * n[i]).sum();
            if dot > 0.0 {  // only reflect if moving outward
                for i in 0..N { vel[i] -= (1.0 + self.restitution) * dot * n[i]; }
            }
            // Reconstruct pos_old so next Verlet step uses reflected velocity.
            for i in 0..N { pos_old[i] = pos[i] - vel[i]; }
        }
    }
}

/// Clamp a coordinate to `[min, max]`, reflecting implicit velocity on contact.
/// Reconstructs `pos_old` so the next Verlet step produces the reflected velocity.
/// `restitution` ∈ `[0.0, 1.0]` — `1.0` = perfectly elastic, `0.0` = fully inelastic.
#[inline(always)]
fn apply_axis_constraint(min: f64, max: f64, restitution: f64, pos: &mut f64, pos_old: &mut f64) {
    let vel = *pos - *pos_old;

    if *pos < min {
        // ONLY bounce if moving left (escaping to the left)
        if vel < 0.0 {
            *pos = min;
            *pos_old = min + vel * restitution; 
        }
        // If vel >= 0.0, the particle is entering from the left, do nothing!
        
    } else if *pos > max {
        // ONLY bounce if moving right (escaping to the right)
        if vel > 0.0 {
            *pos = max;
            *pos_old = max + vel * restitution; 
        }
        // If vel <= 0.0, the particle is entering from the right, do nothing!
    }
}

/// Clamp a particle to `[min, max]` along one dimension, reflecting implicit velocity on contact.
/// Reconstructs `pos_old` so the next Verlet step produces the reflected velocity.
/// `restitution` ∈ `[0.0, 1.0]` — `1.0` = perfectly elastic, `0.0` = fully inelastic.
pub struct AxisConstraint {
    pub min: f64,
    pub max: f64,
    pub restitution: f64,
}
impl AxisConstraint {
    pub fn new(min: f64, max: f64, restitution: f64) -> Self { Self { min, max, restitution } }
    #[inline(always)]
    pub fn apply(&self, pos: &mut f64, pos_old: &mut f64) {
        apply_axis_constraint(self.min, self.max, self.restitution, pos, pos_old);
    }
}

/// Clamp a particle inside a rectangle `[x_min, x_max] × [y_min, y_max]`.
/// Reconstructs `pos_old` so the next Verlet step produces reflected velocity on each axis.
/// `restitution` ∈ `[0.0, 1.0]` applies to both axes.
#[derive(Debug, Clone, Copy)] 
pub struct RectConstraint {
    pub min: DVec2,
    pub max: DVec2,
    pub restitution: f64,
}
impl RectConstraint {
    pub fn new<V>(min: V, max: V, restitution: f64) -> Self
    where
        V: Into<DVec2>,
    {  
        Self { 
            min: min.into(),
            max: max.into(),
            restitution
        }
    }

    pub fn from_constraint(other: &RectConstraint, inset: f64, restitution: f64) -> Self {
        let splat = DVec2::splat(inset);
        Self {
            min: other.min + splat,
            max: other.max - splat,
            restitution,
        }
    }


    pub fn from_bounds_with_inset(bounds: &Bounds, inset: f64, restitution: f64) -> Self {
        let splat = DVec2::splat(inset);
        let min_2d = bounds.min.truncate() + splat;
        let max_2d = bounds.max.truncate() - splat;
 
        Self {
            min: min_2d,
            max: max_2d,
            restitution,
        }
    }


    #[inline(always)]
    pub fn apply(&self, x_pos: &mut f64, x_old: &mut f64, y_pos: &mut f64, y_old: &mut f64) {
        apply_axis_constraint(self.min.x, self.max.x, self.restitution, x_pos, x_old);
        apply_axis_constraint(self.min.y, self.max.y, self.restitution, y_pos, y_old);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;
    const DT:  f64 = 0.1;

    // -----------------------------------------------------------------------
    // VerletRectConstraint
    // -----------------------------------------------------------------------

    #[test]
    fn verlet_rect_clamps_both_axes_independently() {
        let c = RectConstraint::new((-1.0, 1.0), (-2.0, 2.0), 1.0);
        let mut x_pos = 1.5;
        let mut x_old = 1.4;  // vel = 0.1 toward +x
        let mut y_pos = -2.3;
        let mut y_old = -2.1;  // vel = -0.2 toward -y
        c.apply(&mut x_pos, &mut x_old, &mut y_pos, &mut y_old);
        
        // x clamped to max, reflected
        assert!((x_pos - 1.0).abs() < EPS);
        assert!((x_old - 0.9).abs() < EPS);  // 1.0 - 0.1*1.0
        
        // y clamped to min, reflected
        assert!((y_pos - (-2.0)).abs() < EPS);
        assert!((y_old - (-1.8)).abs() < EPS);  // -2.0 - (-0.2)*1.0
    }

    #[test]
    fn verlet_rect_inside_bounds_no_effect() {
        let c = RectConstraint::new((-1.0, 1.0), (-2.0, 2.0), 1.0);
        let mut x_pos = 0.5;
        let mut x_old = 0.4;
        let mut y_pos = 1.0;
        let mut y_old = 0.9;
        c.apply(&mut x_pos, &mut x_old, &mut y_pos, &mut y_old);
        
        assert!((x_pos - 0.5).abs() < EPS);
        assert!((x_old - 0.4).abs() < EPS);
        assert!((y_pos - 1.0).abs() < EPS);
        assert!((y_old - 0.9).abs() < EPS);
    }

    #[test]
    fn verlet_rect_corner_collision_reflects_both() {
        let c = RectConstraint::new((0.0, 1.0), (0.0, 1.0), 0.8);
        let mut x_pos = -0.1;
        let mut x_old = 0.0;   // vel = -0.1
        let mut y_pos = 1.2;
        let mut y_old = 1.1;   // vel = 0.1
        c.apply(&mut x_pos, &mut x_old, &mut y_pos, &mut y_old);
        
        assert!((x_pos - 0.0).abs() < EPS);
        assert!((x_old - 0.08).abs() < EPS);  // 0.0 - (-0.1)*0.8
        
        assert!((y_pos - 1.0).abs() < EPS);
        assert!((y_old - 0.92).abs() < EPS);  // 1.0 - 0.1*0.8
    }

    // -----------------------------------------------------------------------
    // VerletLinearDrag
    // -----------------------------------------------------------------------

    #[test]
    fn verlet_drag_reduces_acc_proportional_to_implied_vel() {
        // implied vel = (pos - pos_old) / dt = (1.0 - 0.9) / 0.1 = 1.0
        let drag = LinearDrag::new(2.0);
        let mut acc = 0.0;
        drag.apply(1.0, 0.9, DT, &mut acc);  // acc -= 2.0 * 1.0
        assert!((acc - (-2.0)).abs() < EPS);
    }

    #[test]
    fn verlet_drag_zero_displacement_no_effect() {
        let drag = LinearDrag::new(5.0);
        let mut acc = 10.0;
        drag.apply(1.0, 1.0, DT, &mut acc);  // vel = 0
        assert!((acc - 10.0).abs() < EPS);
    }

    #[test]
    fn verlet_drag_negative_vel_increases_acc() {
        // implied vel = (0.9 - 1.0) / 0.1 = -1.0
        let drag = LinearDrag::new(1.0);
        let mut acc = 0.0;
        drag.apply(0.9, 1.0, DT, &mut acc);  // acc -= 1.0 * (-1.0) = +1.0
        assert!((acc - 1.0).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // VerletDamping
    // -----------------------------------------------------------------------

    #[test]
    fn verlet_damping_scales_displacement() {
        // pos=1.0, pos_old=0.8 → displacement=0.2
        // after: pos_old = 1.0 - 0.2*0.9 = 0.82
        let damp = Damping::new(0.9);
        let mut pos_old = 0.8;
        damp.apply(1.0, &mut pos_old);
        assert!((pos_old - 0.82).abs() < EPS);
    }

    #[test]
    fn verlet_damping_factor_zero_stops_particle() {
        let damp = Damping::new(0.0);
        let mut pos_old = 0.5;
        damp.apply(1.0, &mut pos_old);  // pos_old = 1.0 - 0.5*0 = 1.0
        assert!((pos_old - 1.0).abs() < EPS);
    }

    #[test]
    fn verlet_damping_factor_one_no_change() {
        let damp = Damping::new(1.0);
        let mut pos_old = 0.8;
        damp.apply(1.0, &mut pos_old);  // pos_old = 1.0 - (1.0-0.8)*1 = 0.8
        assert!((pos_old - 0.8).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // VerletDimConstraint
    // -----------------------------------------------------------------------

    #[test]
    fn verlet_dim_below_min_clamps_and_reflects() {
        // pos=-0.1, pos_old=0.1 → vel = -0.2 (moving toward min)
        let c = AxisConstraint::new(0.0, 10.0, 1.0);
        let mut pos     = -0.1;
        let mut pos_old =  0.1;
        c.apply(&mut pos, &mut pos_old);
        assert!((pos - 0.0).abs() < EPS);
        // pos_old = min - vel * restitution = 0.0 - (-0.2)*1.0 = 0.2
        assert!((pos_old - 0.2).abs() < EPS);
    }

    #[test]
    fn verlet_dim_above_max_clamps_and_reflects() {
        // pos=10.2, pos_old=10.0 → vel = 0.2 (moving toward max)
        let c = AxisConstraint::new(0.0, 10.0, 1.0);
        let mut pos     = 10.2;
        let mut pos_old = 10.0;
        c.apply(&mut pos, &mut pos_old);
        assert!((pos - 10.0).abs() < EPS);
        // pos_old = max - vel * restitution = 10.0 - 0.2*1.0 = 9.8
        assert!((pos_old - 9.8).abs() < EPS);
    }

    #[test]
    fn verlet_dim_inelastic_reduces_rebound() {
        // pos=-0.1, pos_old=0.0 → vel = -0.1
        let c = AxisConstraint::new(0.0, 10.0, 0.5);
        let mut pos     = -0.1;
        let mut pos_old =  0.0;
        c.apply(&mut pos, &mut pos_old);
        assert!((pos - 0.0).abs() < EPS);
        // pos_old = 0.0 - (-0.1)*0.5 = 0.05
        assert!((pos_old - 0.05).abs() < EPS);
    }

    #[test]
    fn verlet_dim_inside_bounds_no_effect() {
        let c = AxisConstraint::new(0.0, 10.0, 1.0);
        let mut pos     = 5.0;
        let mut pos_old = 4.9;
        c.apply(&mut pos, &mut pos_old);
        assert!((pos - 5.0).abs() < EPS);
        assert!((pos_old - 4.9).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // VerletBallConstraint
    // -----------------------------------------------------------------------

    #[test]
    fn verlet_ball_inside_no_effect() {
        let c = BallConstraint::new([0.0, 0.0], 5.0, 1.0);
        let mut pos     = [1.0, 0.0];
        let mut pos_old = [0.9, 0.0];
        c.apply(&mut pos, &mut pos_old);
        assert!((pos[0] - 1.0).abs() < EPS);
        assert!((pos_old[0] - 0.9).abs() < EPS);
    }

    #[test]
    fn verlet_ball_outside_clamps_to_surface_elastic() {
        // pos on +x axis at 2.0, implied vel = (2.0-1.9, 0) = (0.1, 0)
        let c = BallConstraint::new([0.0, 0.0], 1.0, 1.0);
        let mut pos     = [2.0, 0.0];
        let mut pos_old = [1.9, 0.0];
        c.apply(&mut pos, &mut pos_old);
        // clamped to radius
        assert!((pos[0] - 1.0).abs() < EPS);
        // reflected vel = (-0.1, 0), so pos_old = pos - vel = (1.0 - (-0.1)) = 1.1
        assert!((pos_old[0] - 1.1).abs() < EPS);
    }

    #[test]
    fn verlet_ball_inelastic_stops_at_surface() {
        // fully inelastic: vel normal component killed → pos_old = pos
        let c = BallConstraint::new([0.0, 0.0], 1.0, 0.0);
        let mut pos     = [2.0, 0.0];
        let mut pos_old = [1.9, 0.0];
        c.apply(&mut pos, &mut pos_old);
        assert!((pos[0] - 1.0).abs() < EPS);
        // vel after = 0 along normal → pos_old[0] = pos[0]
        assert!((pos_old[0] - pos[0]).abs() < EPS);
    }

    #[test]
    fn verlet_ball_moving_inward_not_reflected() {
        // outside but implied vel is toward center — should still clamp position
        // but not reflect (dot < 0)
        let c = BallConstraint::new([0.0, 0.0], 1.0, 1.0);
        let mut pos     = [2.0, 0.0];
        let mut pos_old = [2.1, 0.0];  // moving inward (vel = -0.1)
        c.apply(&mut pos, &mut pos_old);
        // position clamped
        assert!((pos[0] - 1.0).abs() < EPS);
        // velocity unchanged → pos_old = pos - vel = 1.0 - (-0.1) = 1.1
        assert!((pos_old[0] - 1.1).abs() < EPS);
    }
}
