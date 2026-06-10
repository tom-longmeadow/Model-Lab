// /// Squared speed from 2 velocity components.
// /// Lower = slower = removed first by `slowest_n`.
// pub fn vel_sq_2d(vx: f64, vy: f64) -> f64 {
//     vx * vx + vy * vy
// }

// /// Squared speed from 3 velocity components.
// pub fn vel_sq_3d(vx: f64, vy: f64, vz: f64) -> f64 {
//     vx * vx + vy * vy + vz * vz
// }

// /// Squared displacement from 2 position/position-old pairs.
// /// Used by Verlet creators — `|pos - pos_old|²` is proportional to speed² × dt².
// /// Lower = more at-rest = removed first by `slowest_n`.
// pub fn displacement_sq_2d(x: f64, x_old: f64, y: f64, y_old: f64) -> f64 {
//     let dx = x - x_old;
//     let dy = y - y_old;
//     dx * dx + dy * dy
// }

// /// Squared displacement from 3 position/position-old pairs.
// pub fn displacement_sq_3d(x: f64, x_old: f64, y: f64, y_old: f64, z: f64, z_old: f64) -> f64 {
//     let dx = x - x_old;
//     let dy = y - y_old;
//     let dz = z - z_old;
//     dx * dx + dy * dy + dz * dz
// }

// // ---------------------------------------------------------------------------
// // Tests
// // ---------------------------------------------------------------------------

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn vel_sq_2d_correct() {
//         assert_eq!(vel_sq_2d(3.0, 4.0), 25.0);
//         assert_eq!(vel_sq_2d(0.0, 0.0), 0.0);
//     }

//     #[test]
//     fn vel_sq_3d_correct() {
//         assert_eq!(vel_sq_3d(1.0, 2.0, 2.0), 9.0);
//     }

//     #[test]
//     fn displacement_sq_2d_correct() {
//         assert_eq!(displacement_sq_2d(3.0, 1.0, 4.0, 1.0), 4.0 + 9.0);
//     }

//     #[test]
//     fn displacement_sq_3d_correct() {
//         assert_eq!(displacement_sq_3d(1.0, 0.0, 2.0, 0.0, 3.0, 0.0), 1.0 + 4.0 + 9.0);
//     }
// }
