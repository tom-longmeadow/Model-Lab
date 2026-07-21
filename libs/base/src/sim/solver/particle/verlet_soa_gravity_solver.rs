use std::hash::Hash; 
use crate::math::{FloatScalar, Vector}; 
use crate::sim::solver::Solver;  
use crate::sim::solver::particle::environment::ParticleEnvironment;  
use crate::sim::solver::particle::flags::CollisionFlags;
use crate::sim::solver::particle::physics::verlet_soa_collision::VerletSoaCollision;
use crate::sim::solver::particle::physics::verlet_soa_constraint::VerletSoaConstraint;
use crate::sim::solver::particle::physics::verlet_soa_kinetics::VerletSoaKinetics;
use crate::sim::solver::particle::physics::verlet_soa_prestep::VerletSoaPrestep; 
use crate::sim::solver::particle::space::collision_registry::CollisionRegistry;
use crate::sim::solver::particle::space::grid_key::GridKey; 
use crate::sim::solver::particle::verlet_soa_vec_storage::{ 
    AccField,  SoaInvMass, SoaPos, SoaPosOld, SoaRadius, VerletParticleSoaVecStorage};
use crate::sim::storage::Storage; 
 


pub struct VerletSoaGravitySolver{
    pub collision_registry: CollisionRegistry,
}

impl VerletSoaGravitySolver { 
    #[inline]
    pub fn new() -> Self {
        Self { 
            collision_registry: CollisionRegistry::with_capacity(2048),
        }
    }
}

impl<V, F> Solver<VerletParticleSoaVecStorage<V>, ParticleEnvironment<V, F>> for VerletSoaGravitySolver
where
    V: Vector + 'static,
     V::Quantized: Hash + Eq + Copy + GridKey,
    F: CollisionFlags + 'static, // Bind to the static engine flag configurations
{
    fn init(
        &mut self, 
        _storage: &mut VerletParticleSoaVecStorage<V>, 
        _environment: &mut ParticleEnvironment<V, F>
    ) {
        self.collision_registry = CollisionRegistry::new(); 
    }

    fn pre_step(
        &mut self, 
        storage: &mut VerletParticleSoaVecStorage<V>, 
        _tick: u64, 
        _step_dt: f64, 
        environment: &mut ParticleEnvironment<V, F>
    ) {
        let len = storage.len();
        if len == 0 { return; }

       
        let view = storage.view_mut(); 

        // 2. Call .slice directly on the view!
        let radii = view.slice(SoaRadius); 

        VerletSoaPrestep::update_grid_cell_size(&radii, environment);


        // // Unpack your runtime bit-masked layout views safely and sequentially
        // let view = storage.view_mut();
        // let mut pos      = view.slice_mut(SoaPos);
        // let mut pos_old  = view.slice_mut(SoaPosOld);
        // let mut color    = view.slice_mut(SoaColor);
          
        // let min_speed = V::Scalar::from_f64(1.0);
        // let max_speed = V::Scalar::from_f64(200.0);
        // let v_dt = V::Scalar::from_f64(step_dt); 

        // VerletSoaPrestep::update_color_from_velocity(
        //     min_speed, max_speed, &mut pos, &mut pos_old, &mut color, v_dt, tick, environment
        // );  
    }
    
    fn sub_step(
        &mut self, 
        storage: &mut VerletParticleSoaVecStorage<V>, 
        sub_step_dt: f64, 
        environment: &mut ParticleEnvironment<V, F>
    ) {
        let len = storage.len();
        if len == 0 { return; }

        let v_dt = V::Scalar::from_f64(sub_step_dt);
        
        // Create exactly ONE view descriptor over the storage engine
        let view = storage.view_mut();

        // 1. Extract shared runtime loans safely
        let radii    = view.slice(SoaRadius);
        let inv_mass = view.slice(SoaInvMass);

        // 2. Extract exclusive mutable loans safely
        let mut pos     = view.slice_mut_typed(SoaPos);
        let mut pos_old = view.slice_mut_typed(SoaPosOld);
        let mut acc     = view.slice_mut_typed(AccField);


   
        // 3. Run your physics modifications safely
        VerletSoaKinetics::apply_uniform_acceleration(&mut acc, environment);
        
        VerletSoaKinetics::update_kinetics(
            &mut pos, 
            &mut pos_old, 
            &mut acc, 
            v_dt, 
            environment
        ); 
        VerletSoaCollision::populate_grid(&pos, environment);

        VerletSoaCollision::resolve_collisions::<V, F>(
            &mut pos,  
            &inv_mass, 
            &radii,  
            &mut self.collision_registry,
            environment
        );

        VerletSoaCollision::apply_particle_restitution::<V, F>(
            &self.collision_registry,
            &pos,          // FIX 1: Pass as shared reference &[V], not &mut
            &mut pos_old,  // Matches &mut [V]
            &inv_mass,     // Matches &[V::Scalar]
            &radii,        // Matches &[V::Scalar]
            environment,
        );

        
        VerletSoaConstraint::apply_bounds(
            &mut pos, 
            &mut pos_old, 
            &radii, 
            v_dt, 
            environment
        );



    }
 
    fn post_step(
        &mut self, 
        _storage: &mut VerletParticleSoaVecStorage<V>,  
        _environment: &ParticleEnvironment<V, F>
    ) {}
}


// pub struct VerletSoaGravitySolver;
// impl<V> Solver<VerletParticleSoaVecStorage<V>, ParticleEnvironment<V>> for VerletSoaGravitySolver
// where
//     V: Vector + 'static,
//     V::Quantized: Hash + Eq + Copy + Ord,  
// {
//     fn init(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>, _environment: &mut ParticleEnvironment<V>) { }

//     fn pre_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, tick: u64, step_dt: f64, environment: &mut ParticleEnvironment<V>) {

// let view = storage.view_mut();
//          let mut positions     = view.slice_mut(PosField);
//     let mut old_positions = view.slice_mut(PosOldField);
//     let accelerations     = view.as_view().slice(AccField); // Read-only is fine!


//         // VerletSoaPrestep::update_jitter(tick, environment);

//         // let len = Storage::len(storage);
//         // if len == 0 { return; }

//         // let cols = storage.columns_mut(); 
//         // let radii = cols.slice(VerletParticleColumns::Radius, len);
//         // let pos   = cols.slice(VerletParticleColumns::Pos, len);
//         // let pos_old   = cols.slice(VerletParticleColumns::PosOld, len);
//         // let color = cols.slice_mut(VerletParticleColumns::Color, len);
        
//         // VerletSoaPrestep::update_grid_cell_size(radii, tick, environment);

//         // let min_speed = V::Scalar::from_f64(1.0);
//         // let max_speed = V::Scalar::from_f64(200.0);
//         // let v_dt = V::Scalar::from_f64(step_dt); 

//         // VerletSoaPrestep::update_color_from_velocity(
//         //     min_speed, max_speed, pos, pos_old, color, v_dt, tick, environment
//         // );  
        
//     }
    
//     fn sub_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, sub_step_dt: f64, environment: &mut ParticleEnvironment<V>) 
//     {

//         // let len = Storage::len(storage);
//         // if len == 0 { return; }

//         // let v_dt = V::Scalar::from_f64(sub_step_dt);

//         // let cols = storage.columns_mut();  
//         // let acc   = cols.slice_mut(VerletParticleColumns::Acc, len);
//         // let pos   = cols.slice_mut(VerletParticleColumns::Pos, len);
//         // let pos_old   = cols.slice_mut(VerletParticleColumns::PosOld, len);
//         // let inv_mass   = cols.slice(VerletParticleColumns::InvMass, len);
//         // let radii   = cols.slice(VerletParticleColumns::Radius, len);

//         // VerletSoaKinetics::apply_uniform_acceleration(acc, v_dt, environment);

        
//         // VerletSoaKinetics::update_kinetics(
//         //     pos, 
//         //     pos_old, 
//         //     acc, 
//         //     v_dt, 
//         //     environment
//         // );

//         // VerletSoaCollision::populate_grid(pos, environment);

//         // VerletSoaCollision::resolve_collisions::<V, A, true, true, true, true>(
//         //         pos, pos_old, inv_mass,  radii,  v_dt,  environment);

 
       
//         // VerletPhysics::soa_apply_position_constraints(
//         //     positions, 
//         //     positions_old, 
//         //     radii, 
//         //     v_dt, 
//         //     environment
//         // );
//     }
 
//     fn post_step(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>,  _environment: &ParticleEnvironment<V>) { }
// }
