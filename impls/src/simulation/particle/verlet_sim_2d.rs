use base::sim::{
    lifecycle::particle::{
        creator::volume::VolumeCreator,
        deletor::slowest::SlowestDeletor,
        lifecycle::{aos::AosLifecycle, soa::SoaLifecycle},
    },
    simulation::Simulation, storage::CpuStorage,
};
use super::particle_2d::{AosStorage2d, SoaStorage2d, VerletParticle2d};
use super::step_model_2d::BoxModel2d;
use super::verlet_solver_2d::{AosVerletSolver2d, SoaVerletSolver2d};

// ---------------------------------------------------------------------------
// Type aliases — name the concrete simulation types
// ---------------------------------------------------------------------------

/// AoS 2D Verlet simulation with volume-based creation and slowest-particle deletion.
///
/// `SF` — spawn closure  (`fn() -> VerletParticle2d`)
/// `SC` — score closure  (`fn(&AosStorage2d) -> Vec<f64>`)
pub type AosVerletSim2d<SF, SC> = Simulation<
    AosStorage2d,
    AosVerletSolver2d,
    AosLifecycle<AosStorage2d, VolumeCreator, SlowestDeletor, SF, SC>,
>;

/// SoA 2D Verlet simulation with volume-based creation and slowest-particle deletion.
///
/// `SF` — spawn closure  (`fn() -> VerletParticle2d`)
/// `SC` — score closure  (`fn(&SoaStorage2d) -> Vec<f64>`)
pub type SoaVerletSim2d<SF, SC> = Simulation<
    SoaStorage2d,
    SoaVerletSolver2d,
    SoaLifecycle<SoaStorage2d, VolumeCreator, SlowestDeletor, SF, SC>,
>;

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

/// Build an AoS 2D Verlet simulation with volume-based lifecycle management.
///
/// # Arguments
/// - `hz`          — ticks per second
/// - `model`       — [`BoxModel2d`] (or any other step model)
/// - `fill_ratio`  — target fill fraction (e.g. 0.8)
/// - `particle_vol`— volume of one particle
/// - `volume_fn`   — closure returning current bounds volume
/// - `spawn`       — closure producing a new [`VerletParticle2d`]
/// - `score_fn`    — closure scoring each particle (lower score = removed first)
pub fn new_aos_verlet_sim_2d<VF, SF, SC>(
    hz:           f64,
    model:        BoxModel2d,
    fill_ratio:   f64,
    particle_vol: f64,
    volume_fn:    VF,
    spawn:        SF,
    score_fn:     SC,
) -> AosVerletSim2d<SF, SC>
where
    VF: Fn() -> f64 + 'static,
    SF: Fn() -> VerletParticle2d + 'static,
    SC: Fn(&AosStorage2d) -> Vec<f64> + 'static,
{
    let solver  = AosVerletSolver2d::new(model);
    let creator = VolumeCreator::new(volume_fn, fill_ratio, particle_vol);
    let deletor = SlowestDeletor;
    let lifecycle = AosLifecycle::new(creator, deletor, spawn, score_fn);
    let storage = AosStorage2d::new(64);
    Simulation::new(hz, storage, solver, lifecycle)
}

/// Build a SoA 2D Verlet simulation with volume-based lifecycle management.
///
/// # Arguments
/// - `hz`          — ticks per second
/// - `model`       — [`BoxModel2d`] (or any other step model)
/// - `fill_ratio`  — target fill fraction (e.g. 0.8)
/// - `particle_vol`— volume of one particle
/// - `volume_fn`   — closure returning current bounds volume
/// - `spawn`       — closure producing a new [`VerletParticle2d`] (decomposed into columns)
/// - `score_fn`    — closure scoring each particle (lower score = removed first)
pub fn new_soa_verlet_sim_2d<VF, SF, SC>(
    hz:           f64,
    model:        BoxModel2d,
    fill_ratio:   f64,
    particle_vol: f64,
    volume_fn:    VF,
    spawn:        SF,
    score_fn:     SC,
) -> SoaVerletSim2d<SF, SC>
where
    VF: Fn() -> f64 + 'static,
    SF: Fn() -> VerletParticle2d + 'static,
    SC: Fn(&SoaStorage2d) -> Vec<f64> + 'static,
{
    let solver  = SoaVerletSolver2d::new(model);
    let creator = VolumeCreator::new(volume_fn, fill_ratio, particle_vol);
    let deletor = SlowestDeletor;
    let lifecycle = SoaLifecycle::new(creator, deletor, spawn, score_fn);
    let storage = SoaStorage2d::new(64);
    Simulation::new(hz, storage, solver, lifecycle)
}




// use base::sim::lifecycle::particle::creator::volume::VolumeCreator;
// use base::sim::{
//     simulation::Simulation, 
// };
// use super::particle_2d::{AosStorage2d, SoaStorage2d, VerletParticle2d};
// use super::step_model_2d::BoxModel2d;
// use super::verlet_solver_2d::{AosVerletSolver2d, SoaVerletSolver2d};

// // ---------------------------------------------------------------------------
// // Closure type aliases — name the concrete creator generics
// // ---------------------------------------------------------------------------

// /// AoS 2D Verlet simulation with a [`VolumeCreator`].
// ///
// /// `VF` — volume closure (`fn() -> f64`)
// /// `SF` — spawn closure  (`fn() -> VerletParticle2d`)
// /// `SC` — score closure  (`fn(&AosStorage2d) -> Vec<f64>`)
// pub type AosVerletSim2d<VF, SF, SC> = Simulation<
//     AosStorage2d,
//     AosVerletSolver2d,
//     VolumeCreator<AosStorage2d, VF, SF, SC>,
// >;

// /// SoA 2D Verlet simulation with a [`VolumeCreator`].
// pub type SoaVerletSim2d<VF, SF, SC> = Simulation<
//     SoaStorage2d,
//     SoaVerletSolver2d,
//     VolumeCreator<SoaStorage2d, VF, SF, SC>,
// >;

// // ---------------------------------------------------------------------------
// // Convenience constructors
// // ---------------------------------------------------------------------------

// /// Build an AoS 2D Verlet simulation.
// ///
// /// # Arguments
// /// - `hz`          — ticks per second
// /// - `model`       — [`BoxModel2d`] (or any other step model, but type is fixed here)
// /// - `fill_ratio`  — target fill fraction passed to [`VolumeCreator`]
// /// - `particle_vol`— per-particle volume used to compute target count
// /// - `volume_fn`   — closure returning current bounds volume
// /// - `spawn`       — closure producing a new [`VerletParticle2d`]
// /// - `score_fn`    — closure scoring each particle (lower = removed first)
// pub fn new_aos_verlet_sim_2d<VF, SF, SC>(
//     hz:           f64,
//     model:        BoxModel2d,
//     fill_ratio:   f64,
//     particle_vol: f64,
//     volume_fn:    VF,
//     spawn:        SF,
//     score_fn:     SC,
// ) -> AosVerletSim2d<VF, SF, SC>
// where
//     VF: Fn() -> f64,
//     SF: Fn() -> VerletParticle2d,
//     SC: Fn(&AosStorage2d) -> Vec<f64>,
// {
//     let solver  = AosVerletSolver2d::new(model);
//     let creator = VolumeCreator::new(volume_fn, fill_ratio, particle_vol, spawn, score_fn);
//     let storage = AosStorage2d::new(64);
//     Simulation::new(hz, storage, solver, creator)
// }

// /// Build a SoA 2D Verlet simulation.
// pub fn new_soa_verlet_sim_2d<VF, SF, SC>(
//     hz:           f64,
//     model:        BoxModel2d,
//     fill_ratio:   f64,
//     particle_vol: f64,
//     volume_fn:    VF,
//     spawn:        SF,
//     score_fn:     SC,
// ) -> SoaVerletSim2d<VF, SF, SC>
// where
//     VF: Fn() -> f64,
//     SF: Fn() -> VerletParticle2d,
//     SC: Fn(&SoaStorage2d) -> Vec<f64>,
// {
//     let solver  = SoaVerletSolver2d::new(model);
//     let creator = VolumeCreator::new(volume_fn, fill_ratio, particle_vol, spawn, score_fn);
//     let storage = SoaStorage2d::new(64);
//     Simulation::new(hz, storage, solver, creator)
// }
