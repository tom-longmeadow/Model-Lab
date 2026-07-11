
 

/// `acc -= coeff * vel` — viscous drag for one component.
/// Requires explicit stored velocity (Newtonian / Leapfrog / VelocityVerlet).
pub struct LinearDrag {
    pub coeff: f64,
}
impl LinearDrag {
    pub fn new(coeff: f64) -> Self { Self { coeff } }
    #[inline(always)]
    pub fn apply(&self, vel: f64, acc: &mut f64) {
        *acc -= self.coeff * vel;
    }
}

/// `vel *= factor` — velocity damping for one component.
/// Requires explicit stored velocity (Newtonian / Leapfrog / VelocityVerlet).
pub struct Damping {
    pub factor: f64,
}
impl Damping {
    pub fn new(factor: f64) -> Self { Self { factor } }
    #[inline(always)]
    pub fn apply(&self, vel: &mut f64) {
        *vel *= self.factor;
    }
}

/// Clamp a particle inside a ball (circle in 2D, sphere in 3D) centred at `center` with `radius`.
/// Reflects velocity off the surface normal on contact.
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
    pub fn apply(&self, pos: &mut [f64; N], vel: &mut [f64; N]) {
        let mut delta = [0.0f64; N];
        let mut dist2 = 0.0f64;
        for i in 0..N {
            delta[i] = pos[i] - self.center[i];
            dist2 += delta[i] * delta[i];
        }
        let r2 = self.radius * self.radius;
        if dist2 > r2 {
            let dist  = dist2.sqrt();
            let inv   = 1.0 / dist;
            let mut n = [0.0f64; N];
            for i in 0..N { n[i] = delta[i] * inv; }
            for i in 0..N { pos[i] = self.center[i] + n[i] * self.radius; }
            let dot: f64 = (0..N).map(|i| vel[i] * n[i]).sum();
            if dot > 0.0 {
                for i in 0..N { vel[i] -= (1.0 + self.restitution) * dot * n[i]; }
            }
        }
    }
}

// /// Clamp a coordinate to `[min, max]`, reflecting velocity on contact.
// /// `restitution` ∈ `[0.0, 1.0]` — `1.0` = perfectly elastic, `0.0` = fully inelastic.
// #[inline(always)]
// fn apply_axis_constraint(min: f64, max: f64, restitution: f64, pos: &mut f64, vel: &mut f64) {
//     if *pos < min {
//         *pos = min;
//         *vel = -(*vel) * restitution;  // reflect velocity
//     } else if *pos > max {
//         *pos = max;
//         *vel = -(*vel) * restitution;
//     }
// }

// /// Clamp a particle to `[min, max]` along one dimension, reflecting velocity on contact.
// /// `restitution` ∈ `[0.0, 1.0]` — `1.0` = perfectly elastic, `0.0` = fully inelastic.
// pub struct AxisConstraint {
//     pub min: f64,
//     pub max: f64,
//     pub restitution: f64,
// }
// impl AxisConstraint {
//     pub fn new(min: f64, max: f64, restitution: f64) -> Self { Self { min, max, restitution } }
//     #[inline(always)]
//     pub fn apply(&self, pos: &mut f64, vel: &mut f64) {
//         apply_axis_constraint(self.min, self.max, self.restitution, pos, vel);
//     }
// }

// /// Clamp a particle inside a rectangle `[x_min, x_max] × [y_min, y_max]`.
// /// Reflects velocity on contact with each boundary independently.
// /// `restitution` ∈ `[0.0, 1.0]` applies to both axes.
// pub struct RectConstraint {
//     pub x_min: f64,
//     pub x_max: f64,
//     pub y_min: f64,
//     pub y_max: f64,
//     pub restitution: f64,
// }
// impl RectConstraint {
//     pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64, restitution: f64) -> Self {
//         Self { x_min, x_max, y_min, y_max, restitution }
//     }

//     #[inline(always)]
//     pub fn apply(&self, x_pos: &mut f64, x_vel: &mut f64, y_pos: &mut f64, y_vel: &mut f64) {
//         apply_axis_constraint(self.x_min, self.x_max, self.restitution, x_pos, x_vel);
//         apply_axis_constraint(self.y_min, self.y_max, self.restitution, y_pos, y_vel);
//     }
// }

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

     // -----------------------------------------------------------------------
    // NewtonianRectConstraint
    // -----------------------------------------------------------------------

    // #[test]
    // fn newtonian_rect_clamps_both_axes_independently() {
    //     let c = RectConstraint::new(-1.0, 1.0, -2.0, 2.0, 1.0);
    //     let mut x_pos = 1.5;
    //     let mut x_vel = 2.0;   // moving toward +x
    //     let mut y_pos = -2.3;
    //     let mut y_vel = -3.0;  // moving toward -y
    //     c.apply(&mut x_pos, &mut x_vel, &mut y_pos, &mut y_vel);
        
    //     // x clamped to max, vel reflected
    //     assert!((x_pos - 1.0).abs() < EPS);
    //     assert!((x_vel - (-2.0)).abs() < EPS);
        
    //     // y clamped to min, vel reflected
    //     assert!((y_pos - (-2.0)).abs() < EPS);
    //     assert!((y_vel - 3.0).abs() < EPS);
    // }

    // #[test]
    // fn newtonian_rect_inside_bounds_no_effect() {
    //     let c = RectConstraint::new(-1.0, 1.0, -2.0, 2.0, 1.0);
    //     let mut x_pos = 0.5;
    //     let mut x_vel = 1.0;
    //     let mut y_pos = 1.0;
    //     let mut y_vel = 0.5;
    //     c.apply(&mut x_pos, &mut x_vel, &mut y_pos, &mut y_vel);
        
    //     assert!((x_pos - 0.5).abs() < EPS);
    //     assert!((x_vel - 1.0).abs() < EPS);
    //     assert!((y_pos - 1.0).abs() < EPS);
    //     assert!((y_vel - 0.5).abs() < EPS);
    // }

    // #[test]
    // fn newtonian_rect_corner_collision_reflects_both() {
    //     let c = RectConstraint::new(0.0, 1.0, 0.0, 1.0, 0.6);
    //     let mut x_pos = -0.1;
    //     let mut x_vel = -5.0;
    //     let mut y_pos = 1.2;
    //     let mut y_vel = 4.0;
    //     c.apply(&mut x_pos, &mut x_vel, &mut y_pos, &mut y_vel);
        
    //     assert!((x_pos - 0.0).abs() < EPS);
    //     assert!((x_vel - 3.0).abs() < EPS);  // 5.0 * 0.6
        
    //     assert!((y_pos - 1.0).abs() < EPS);
    //     assert!((y_vel - (-2.4)).abs() < EPS);  // -4.0 * 0.6
    // }

    // -----------------------------------------------------------------------
    // NewtonianLinearDrag
    // -----------------------------------------------------------------------

    #[test]
    fn linear_drag_reduces_acc_proportional_to_vel() {
        let drag = LinearDrag::new(2.0);
        let mut acc = 0.0;
        drag.apply(3.0, &mut acc);  // acc -= 2.0 * 3.0
        assert!((acc - (-6.0)).abs() < EPS);
    }

    #[test]
    fn linear_drag_zero_vel_no_effect() {
        let drag = LinearDrag::new(5.0);
        let mut acc = 10.0;
        drag.apply(0.0, &mut acc);
        assert!((acc - 10.0).abs() < EPS);
    }

    #[test]
    fn linear_drag_negative_vel_increases_acc() {
        let drag = LinearDrag::new(1.0);
        let mut acc = 0.0;
        drag.apply(-4.0, &mut acc);  // acc -= 1.0 * (-4.0) = +4.0
        assert!((acc - 4.0).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // NewtonianDamping
    // -----------------------------------------------------------------------

    #[test]
    fn damping_scales_velocity() {
        let damp = Damping::new(0.9);
        let mut vel = 10.0;
        damp.apply(&mut vel);
        assert!((vel - 9.0).abs() < EPS);
    }

    #[test]
    fn damping_factor_zero_stops_particle() {
        let damp = Damping::new(0.0);
        let mut vel = 100.0;
        damp.apply(&mut vel);
        assert!((vel).abs() < EPS);
    }

    #[test]
    fn damping_factor_one_no_change() {
        let damp = Damping::new(1.0);
        let mut vel = 7.5;
        damp.apply(&mut vel);
        assert!((vel - 7.5).abs() < EPS);
    }

    // -----------------------------------------------------------------------
    // NewtonianDimConstraint
    // -----------------------------------------------------------------------

    //#[test]
    // fn dim_constraint_below_min_clamps_and_reflects() {
    //     let c = AxisConstraint::new(0.0, 10.0, 1.0);
    //     let mut pos = -1.0;
    //     let mut vel = -5.0;
    //     c.apply(&mut pos, &mut vel);
    //     assert!((pos - 0.0).abs() < EPS);
    //     assert!((vel - 5.0).abs() < EPS);  // reflected, elastic
    // }

    // #[test]
    // fn dim_constraint_above_max_clamps_and_reflects() {
    //     let c = AxisConstraint::new(0.0, 10.0, 1.0);
    //     let mut pos = 11.0;
    //     let mut vel = 3.0;
    //     c.apply(&mut pos, &mut vel);
    //     assert!((pos - 10.0).abs() < EPS);
    //     assert!((vel - (-3.0)).abs() < EPS);
    // }

    // #[test]
    // fn dim_constraint_inelastic_loses_speed() {
    //     let c = AxisConstraint::new(0.0, 10.0, 0.5);
    //     let mut pos = -1.0;
    //     let mut vel = -4.0;
    //     c.apply(&mut pos, &mut vel);
    //     assert!((pos - 0.0).abs() < EPS);
    //     assert!((vel - 2.0).abs() < EPS);  // 4.0 * 0.5
    // }

    // #[test]
    // fn dim_constraint_inside_bounds_no_effect() {
    //     let c = AxisConstraint::new(0.0, 10.0, 1.0);
    //     let mut pos = 5.0;
    //     let mut vel = 2.0;
    //     c.apply(&mut pos, &mut vel);
    //     assert!((pos - 5.0).abs() < EPS);
    //     assert!((vel - 2.0).abs() < EPS);
    // }

    // -----------------------------------------------------------------------
    // NewtonianBallConstraint
    // -----------------------------------------------------------------------

    #[test]
    fn ball_constraint_inside_no_effect() {
        let c = BallConstraint::new([0.0, 0.0], 5.0, 1.0);
        let mut pos = [1.0, 0.0];
        let mut vel = [1.0, 0.0];
        c.apply(&mut pos, &mut vel);
        assert!((pos[0] - 1.0).abs() < EPS);
        assert!((vel[0] - 1.0).abs() < EPS);
    }

    #[test]
    fn ball_constraint_outside_clamps_to_surface() {
        let c = BallConstraint::new([0.0, 0.0], 1.0, 1.0);
        let mut pos = [2.0, 0.0];  // outside, exactly on x-axis
        let mut vel = [1.0, 0.0];  // moving outward
        c.apply(&mut pos, &mut vel);
        // position clamped to radius
        assert!((pos[0] - 1.0).abs() < EPS);
        assert!((pos[1]).abs() < EPS);
        // velocity fully reflected (elastic), moving inward
        assert!((vel[0] - (-1.0)).abs() < EPS);
        assert!((vel[1]).abs() < EPS);
    }

    #[test]
    fn ball_constraint_inelastic_reduces_speed() {
        let c = BallConstraint::new([0.0, 0.0], 1.0, 0.0);
        let mut pos = [2.0, 0.0];
        let mut vel = [2.0, 0.0];  // moving outward
        c.apply(&mut pos, &mut vel);
        // fully inelastic — normal component killed
        assert!((vel[0]).abs() < EPS);
    }

    #[test]
    fn ball_constraint_moving_inward_not_reflected() {
        // Particle is outside but moving toward center — no reflection
        let c = BallConstraint::new([0.0, 0.0], 1.0, 1.0);
        let mut pos = [2.0, 0.0];
        let mut vel = [-1.0, 0.0];  // moving inward
        c.apply(&mut pos, &mut vel);
        // position still clamped
        assert!((pos[0] - 1.0).abs() < EPS);
        // velocity unchanged (dot < 0, no reflection)
        assert!((vel[0] - (-1.0)).abs() < EPS);
    }
}
