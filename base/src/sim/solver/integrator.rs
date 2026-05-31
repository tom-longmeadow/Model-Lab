
// ---------------------------------------------------------------------------
// Integrators — pure functions over data.
// No storage, no traits, no heap. Just math on references or slices.
// scalar form: one item   at a time → AosSolver (compiler inlines per item)
// slice  form: one column at a time → SoaSolver (compiler can auto-vectorize / SIMD)
//
// SoA slice layout: blocked — all x values, then all y values etc.
// i.e. pos = [x0, x1, x2, ..., y0, y1, y2, ...]  not interleaved.
//
// All functions take references — zero copying, data modified in place.
// ---------------------------------------------------------------------------

/// Newtonian (velocity) Verlet: `vel += acc * dt`, `pos += vel * dt`.
/// Requires [`NewtonianParticle<N>`] — needs explicit velocity.
pub struct SymplecticEuler;
impl SymplecticEuler {
    #[inline(always)]
    pub fn step_scalar<const N: usize>(
        pos: &mut [f64; N], vel: &mut [f64; N], acc: &[f64; N], dt: f64,
    ) {
        for i in 0..N {
            vel[i] += acc[i] * dt;
            pos[i] += vel[i] * dt;
        }
    }

    #[inline(always)]
    pub fn step_slice(pos: &mut [f64], vel: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..pos.len() {
            vel[i] += acc[i] * dt;
            pos[i] += vel[i] * dt;
        }
    }
}

/// Classic (Störmer) Verlet: `pos_new = 2*pos - pos_old + acc * dt²`.
/// Requires [`VerletParticle<N>`] — needs previous position.
/// Velocity is implicit in the position difference.
pub struct Verlet;
impl Verlet {
    #[inline(always)]
    pub fn step_scalar<const N: usize>(
        pos: &mut [f64; N], pos_old: &mut [f64; N], acc: &[f64; N], dt: f64,
    ) {
        for i in 0..N {
            let new    = 2.0 * pos[i] - pos_old[i] + acc[i] * dt * dt;
            pos_old[i] = pos[i];
            pos[i]     = new;
        }
    }

    #[inline(always)]
    pub fn step_slice(pos: &mut [f64], pos_old: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..pos.len() {
            let new    = 2.0 * pos[i] - pos_old[i] + acc[i] * dt * dt;
            pos_old[i] = pos[i];
            pos[i]     = new;
        }
    }
}

/// Leapfrog (symplectic). Split into half-kick and drift.
/// Requires [`NewtonianParticle<N>`] — needs explicit velocity.
/// Sequence per substep: `half_kick` → `drift` → recompute forces → `half_kick`
/// Conserves energy better than [`SymplecticEuler`] over long runs.
pub struct Leapfrog;
impl Leapfrog {
    #[inline(always)]
    pub fn half_kick_scalar<const N: usize>(vel: &mut [f64; N], acc: &[f64; N], dt: f64) {
        for i in 0..N { vel[i] += acc[i] * dt * 0.5; }
    }

    #[inline(always)]
    pub fn drift_scalar<const N: usize>(pos: &mut [f64; N], vel: &[f64; N], dt: f64) {
        for i in 0..N { pos[i] += vel[i] * dt; }
    }

    #[inline(always)]
    pub fn half_kick_slice(vel: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..vel.len() { vel[i] += acc[i] * dt * 0.5; }
    }

    #[inline(always)]
    pub fn drift_slice(pos: &mut [f64], vel: &[f64], dt: f64) {
        for i in 0..pos.len() { pos[i] += vel[i] * dt; }
    }
}

/// Velocity Verlet (2nd-order, symplectic). Split into two half-kicks around a force recompute.
/// Requires [`NewtonianParticle<N>`] — needs explicit velocity.
///
/// Sequence per substep:
/// 1. `pos += vel·dt + ½·acc·dt²`  and  `vel += ½·acc·dt`   (using *current* acc)
/// 2. recompute forces at new positions
/// 3. `vel += ½·acc_new·dt`                                  (using *new* acc)
///
/// No `acc_old` field needed — the particle's `acc` field is current before
/// `model.pre()` and new afterwards. 2nd-order accuracy; conserves energy
/// over long runs like [`Leapfrog`], but directly yields velocity at each step.
pub struct VelocityVerlet;
impl VelocityVerlet {
    /// Step 1 — position update + first half vel-kick using current `acc`.
    #[inline(always)]
    pub fn step1_scalar<const N: usize>(
        pos: &mut [f64; N], vel: &mut [f64; N], acc: &[f64; N], dt: f64,
    ) {
        for i in 0..N {
            pos[i] += vel[i] * dt + 0.5 * acc[i] * dt * dt;
            vel[i] += 0.5 * acc[i] * dt;
        }
    }

    /// Step 2 — second half vel-kick using freshly recomputed `acc`.
    #[inline(always)]
    pub fn step2_scalar<const N: usize>(vel: &mut [f64; N], acc: &[f64; N], dt: f64) {
        for i in 0..N { vel[i] += 0.5 * acc[i] * dt; }
    }

    #[inline(always)]
    pub fn step1_slice(pos: &mut [f64], vel: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..pos.len() {
            pos[i] += vel[i] * dt + 0.5 * acc[i] * dt * dt;
            vel[i] += 0.5 * acc[i] * dt;
        }
    }

    #[inline(always)]
    pub fn step2_slice(vel: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..vel.len() { vel[i] += 0.5 * acc[i] * dt; }
    }
}
