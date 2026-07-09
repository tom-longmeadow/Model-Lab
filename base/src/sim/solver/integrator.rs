
 
/// `vel += acc·dt`,  `pos += vel·dt`
pub struct SymplecticEuler;
impl SymplecticEuler {
    #[inline(always)]
    pub fn step(pos: &mut f64, vel: &mut f64, acc: f64, dt: f64) {
        *vel += acc * dt;
        *pos += *vel * dt;
    }

}

/// `pos_new = 2·pos − pos_old + acc·dt²`,  `pos_old ← pos`,  `pos ← pos_new`
pub struct Verlet;
impl Verlet {
    #[inline(always)]
    pub fn step(pos: &mut f64, pos_old: &mut f64, acc: f64, dt: f64) {
        let new  = 2.0 * *pos - *pos_old + acc * dt * dt;
        *pos_old = *pos;
        *pos     = new;
    }
}

/// Velocity Verlet — split into two half-kicks around a force recompute.
///
/// Sequence per substep (driven by the solver):
///   1. `step1` — `pos += vel·dt + ½·acc·dt²`,  `vel += ½·acc·dt`  (current acc)
///   2. recompute forces at new positions
///   3. `step2` — `vel += ½·acc_new·dt`
pub struct VelocityVerlet;
impl VelocityVerlet {
    #[inline(always)]
    pub fn step1(pos: &mut f64, vel: &mut f64, acc: f64, dt: f64) {
        *pos += *vel * dt + 0.5 * acc * dt * dt;
        *vel += 0.5 * acc * dt;
    }

    #[inline(always)]
    pub fn step2(vel: &mut f64, acc: f64, dt: f64) {
        *vel += 0.5 * acc * dt;
    }

}

/// Symplectic leapfrog — split into half-kick and drift so forces can be
/// recomputed at the mid-point positions.
///
/// Sequence per substep (driven by the solver):
///   1. `half_kick` — `vel += ½·acc·dt`
///   2. `drift`     — `pos += vel·dt`
///   3. recompute forces at new positions
///   4. `half_kick` — `vel += ½·acc_new·dt`
pub struct Leapfrog;
impl Leapfrog {
    #[inline(always)]
    pub fn half_kick(vel: &mut f64, acc: f64, dt: f64) {
        *vel += acc * dt * 0.5;
    }

    #[inline(always)]
    pub fn drift(pos: &mut f64, vel: f64, dt: f64) {
        *pos += vel * dt;
    }
}



// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f64 = 0.1;
    const EPS: f64 = 1e-12;

    // -----------------------------------------------------------------------
    // SymplecticEuler
    // -----------------------------------------------------------------------

    #[test]
    fn symplectic_euler_zero_acc_constant_velocity() {
        let mut pos = 0.0;
        let mut vel = 2.0;
        SymplecticEuler::step(&mut pos, &mut vel, 0.0, DT);
        assert!((vel - 2.0).abs() < EPS);
        assert!((pos - 0.2).abs() < EPS);
    }

    #[test]
    fn symplectic_euler_constant_acc_updates_vel_first() {
        // vel += acc*dt first, then pos += new_vel*dt
        let mut pos = 0.0;
        let mut vel = 0.0;
        SymplecticEuler::step(&mut pos, &mut vel, 10.0, DT);
        assert!((vel - 1.0).abs() < EPS);   // 0 + 10*0.1
        assert!((pos - 0.1).abs() < EPS);   // 0 + 1.0*0.1  (uses updated vel)
    }

    #[test]
    fn symplectic_euler_two_steps_freefall() {
        let mut pos = 0.0;
        let mut vel = 0.0;
        let acc = -9.8;
        SymplecticEuler::step(&mut pos, &mut vel, acc, DT);
        SymplecticEuler::step(&mut pos, &mut vel, acc, DT);
        // step1: vel=-0.98, pos=-0.098
        // step2: vel=-1.96, pos=-0.098 + -1.96*0.1 = -0.294
        assert!((vel - (-1.96)).abs() < EPS);
        assert!((pos - (-0.294)).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // Verlet
    // -----------------------------------------------------------------------

    #[test]
    fn verlet_zero_acc_constant_velocity() {
        // pos_old = -0.1 so implied vel = 0.1/dt = 1.0
        let mut pos     = 0.0;
        let mut pos_old = -0.1;
        Verlet::step(&mut pos, &mut pos_old, 0.0, DT);
        // new = 2*0.0 - (-0.1) + 0 = 0.1
        assert!((pos - 0.1).abs() < EPS);
        assert!((pos_old - 0.0).abs() < EPS);
    }

    #[test]
    fn verlet_constant_acc_freefall() {
        let mut pos     = 0.0;
        let mut pos_old = 0.0;  // particle at rest
        let acc = -9.8;
        Verlet::step(&mut pos, &mut pos_old, acc, DT);
        // new = 0 - 0 + (-9.8)*0.01 = -0.098
        assert!((pos - (-0.098)).abs() < EPS);
        assert!((pos_old - 0.0).abs() < EPS);
    }

    #[test]
    fn verlet_pos_old_is_previous_pos_after_step() {
        let mut pos     = 1.0;
        let mut pos_old = 0.9;
        let prev_pos = pos;
        Verlet::step(&mut pos, &mut pos_old, 0.0, DT);
        assert!((pos_old - prev_pos).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // Leapfrog
    // -----------------------------------------------------------------------

    #[test]
    fn leapfrog_half_kick_adds_half_impulse() {
        let mut vel = 0.0;
        Leapfrog::half_kick(&mut vel, 10.0, DT);
        assert!((vel - 0.5).abs() < EPS);  // 0 + 10 * 0.1 * 0.5
    }

    #[test]
    fn leapfrog_drift_advances_position() {
        let mut pos = 0.0;
        Leapfrog::drift(&mut pos, 3.0, DT);
        assert!((pos - 0.3).abs() < EPS);
    }

    #[test]
    fn leapfrog_full_step_matches_velocity_verlet() {
        // One full leapfrog step under constant acc should match VelocityVerlet.
        let acc = 4.0;
        let (mut lf_pos, mut lf_vel) = (0.0f64, 0.0f64);
        let (mut vv_pos, mut vv_vel) = (0.0f64, 0.0f64);

        // Leapfrog: half_kick → drift → half_kick (constant acc, no recompute needed)
        Leapfrog::half_kick(&mut lf_vel, acc, DT);
        Leapfrog::drift(&mut lf_pos, lf_vel, DT);
        Leapfrog::half_kick(&mut lf_vel, acc, DT);

        // VelocityVerlet: step1 → step2
        VelocityVerlet::step1(&mut vv_pos, &mut vv_vel, acc, DT);
        VelocityVerlet::step2(&mut vv_vel, acc, DT);

        assert!((lf_pos - vv_pos).abs() < EPS);
        assert!((lf_vel - vv_vel).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // VelocityVerlet
    // -----------------------------------------------------------------------

    #[test]
    fn velocity_verlet_step1_updates_pos_and_half_vel() {
        let mut pos = 0.0;
        let mut vel = 0.0;
        VelocityVerlet::step1(&mut pos, &mut vel, 10.0, DT);
        // pos = 0 + 0*0.1 + 0.5*10*0.01 = 0.05
        // vel = 0 + 0.5*10*0.1 = 0.5
        assert!((pos - 0.05).abs() < EPS);
        assert!((vel - 0.5).abs() < EPS);
    }

    #[test]
    fn velocity_verlet_step2_completes_vel_kick() {
        let mut vel = 0.5;
        VelocityVerlet::step2(&mut vel, 10.0, DT);
        assert!((vel - 1.0).abs() < EPS);  // 0.5 + 0.5*10*0.1
    }

    #[test]
    fn velocity_verlet_zero_acc_constant_velocity() {
        let mut pos = 1.0;
        let mut vel = 2.0;
        VelocityVerlet::step1(&mut pos, &mut vel, 0.0, DT);
        VelocityVerlet::step2(&mut vel, 0.0, DT);
        assert!((pos - 1.2).abs() < EPS);
        assert!((vel - 2.0).abs() < EPS);
    }
}
