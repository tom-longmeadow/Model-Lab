pub mod bounds;
pub mod clock;
pub mod creator;
pub mod simulation;
pub mod solver;
pub mod storage; 


// base/src/sim/
//     solver/
//         mod.rs          — Solver trait only
//         particle.rs     — ParticleSolver + ForceModel trait
//         grid.rs         — GridSolver + AdvectionModel + PressureModel trait
//         constraint.rs   — ConstraintSolver + ConstraintModel trait
//         agent.rs        — AgentSolver + BehaviourModel trait
//     solver/aos/
//         verlet.rs
//         leapfrog.rs
//         classic_verlet.rs

// impls/src/sim/
//     force/
//         gravity.rs
//         drag.rs
//     constraint/
//         distance.rs
//     behaviour/
//         flocking.rs