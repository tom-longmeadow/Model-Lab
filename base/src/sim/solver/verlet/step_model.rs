/// `acc -= coeff * (pos - pos_old) / dt` — viscous drag for one component.
/// Velocity is approximated from the displacement; no stored velocity needed.
pub struct VerletLinearDrag {
    pub coeff: f64,
}
impl VerletLinearDrag {
    pub fn new(coeff: f64) -> Self { Self { coeff } }
    #[inline(always)]
    pub fn apply(&self, pos: f64, pos_old: f64, dt: f64, acc: &mut f64) {
        let vel = (pos - pos_old) / dt;
        *acc -= self.coeff * vel;
    }
}

/// Scale the implicit velocity by damping the displacement.
/// `pos_old = pos - (pos - pos_old) * factor` — equivalent to `vel *= factor`.
pub struct VerletDamping {
    pub factor: f64,
}
impl VerletDamping {
    pub fn new(factor: f64) -> Self { Self { factor } }
    #[inline(always)]
    pub fn apply(&self, pos: f64, pos_old: &mut f64) {
        *pos_old = pos - (pos - *pos_old) * self.factor;
    }
}

/// Clamp a particle inside a ball (circle in 2D, sphere in 3D) centred at `center` with `radius`.
/// Reconstructs `pos_old` so the next Verlet step produces the reflected velocity.
/// `restitution` ∈ `[0.0, 1.0]` — `1.0` = perfectly elastic, `0.0` = fully inelastic.
pub struct VerletBallConstraint<const N: usize> {
    pub center:      [f64; N],
    pub radius:      f64,
    pub restitution: f64,
}
impl<const N: usize> VerletBallConstraint<N> {
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

/// Clamp a particle to `[min, max]` along one dimension, reflecting implicit velocity on contact.
/// Reconstructs `pos_old` so the next Verlet step produces the reflected velocity.
/// `restitution` ∈ `[0.0, 1.0]` — `1.0` = perfectly elastic, `0.0` = fully inelastic.
pub struct VerletDimConstraint {
    pub min: f64,
    pub max: f64,
    pub restitution: f64,
}
impl VerletDimConstraint {
    pub fn new(min: f64, max: f64, restitution: f64) -> Self { Self { min, max, restitution } }
    #[inline(always)]
    pub fn apply(&self, pos: &mut f64, pos_old: &mut f64) {
        if *pos < self.min {
            let vel = *pos - *pos_old;          // negative (moving toward min)
            *pos = self.min;
            *pos_old = self.min - vel * self.restitution;  // reflected: pos_old < min → next step moves away
        } else if *pos > self.max {
            let vel = *pos - *pos_old;          // positive (moving toward max)
            *pos = self.max;
            *pos_old = self.max - vel * self.restitution;  // reflected: pos_old > max → next step moves away
        }
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
    // VerletLinearDrag
    // -----------------------------------------------------------------------

    #[test]
    fn verlet_drag_reduces_acc_proportional_to_implied_vel() {
        // implied vel = (pos - pos_old) / dt = (1.0 - 0.9) / 0.1 = 1.0
        let drag = VerletLinearDrag::new(2.0);
        let mut acc = 0.0;
        drag.apply(1.0, 0.9, DT, &mut acc);  // acc -= 2.0 * 1.0
        assert!((acc - (-2.0)).abs() < EPS);
    }

    #[test]
    fn verlet_drag_zero_displacement_no_effect() {
        let drag = VerletLinearDrag::new(5.0);
        let mut acc = 10.0;
        drag.apply(1.0, 1.0, DT, &mut acc);  // vel = 0
        assert!((acc - 10.0).abs() < EPS);
    }

    #[test]
    fn verlet_drag_negative_vel_increases_acc() {
        // implied vel = (0.9 - 1.0) / 0.1 = -1.0
        let drag = VerletLinearDrag::new(1.0);
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
        let damp = VerletDamping::new(0.9);
        let mut pos_old = 0.8;
        damp.apply(1.0, &mut pos_old);
        assert!((pos_old - 0.82).abs() < EPS);
    }

    #[test]
    fn verlet_damping_factor_zero_stops_particle() {
        let damp = VerletDamping::new(0.0);
        let mut pos_old = 0.5;
        damp.apply(1.0, &mut pos_old);  // pos_old = 1.0 - 0.5*0 = 1.0
        assert!((pos_old - 1.0).abs() < EPS);
    }

    #[test]
    fn verlet_damping_factor_one_no_change() {
        let damp = VerletDamping::new(1.0);
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
        let c = VerletDimConstraint::new(0.0, 10.0, 1.0);
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
        let c = VerletDimConstraint::new(0.0, 10.0, 1.0);
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
        let c = VerletDimConstraint::new(0.0, 10.0, 0.5);
        let mut pos     = -0.1;
        let mut pos_old =  0.0;
        c.apply(&mut pos, &mut pos_old);
        assert!((pos - 0.0).abs() < EPS);
        // pos_old = 0.0 - (-0.1)*0.5 = 0.05
        assert!((pos_old - 0.05).abs() < EPS);
    }

    #[test]
    fn verlet_dim_inside_bounds_no_effect() {
        let c = VerletDimConstraint::new(0.0, 10.0, 1.0);
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
        let c = VerletBallConstraint::new([0.0, 0.0], 5.0, 1.0);
        let mut pos     = [1.0, 0.0];
        let mut pos_old = [0.9, 0.0];
        c.apply(&mut pos, &mut pos_old);
        assert!((pos[0] - 1.0).abs() < EPS);
        assert!((pos_old[0] - 0.9).abs() < EPS);
    }

    #[test]
    fn verlet_ball_outside_clamps_to_surface_elastic() {
        // pos on +x axis at 2.0, implied vel = (2.0-1.9, 0) = (0.1, 0)
        let c = VerletBallConstraint::new([0.0, 0.0], 1.0, 1.0);
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
        let c = VerletBallConstraint::new([0.0, 0.0], 1.0, 0.0);
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
        let c = VerletBallConstraint::new([0.0, 0.0], 1.0, 1.0);
        let mut pos     = [2.0, 0.0];
        let mut pos_old = [2.1, 0.0];  // moving inward (vel = -0.1)
        c.apply(&mut pos, &mut pos_old);
        // position clamped
        assert!((pos[0] - 1.0).abs() < EPS);
        // velocity unchanged → pos_old = pos - vel = 1.0 - (-0.1) = 1.1
        assert!((pos_old[0] - 1.1).abs() < EPS);
    }
}
