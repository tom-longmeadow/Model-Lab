use crate::math::Vector;
use crate::prelude::solver::particle::space::GridSpace;
use crate::prelude::solver::particle::tuning::SimulationTuning;
use crate::prelude::solver::particle::state::State;
use crate::sim::simulation::SubstepProvider;
use crate::sim::solver::particle::flags::{CollisionFlags}; 


pub enum GravityModel<V: Vector> {
    Constant(V),                          // Global gravity (e.g., [0, -9.8, 0])
    Zero,                                 // Space/Microgravity
    //PointSource { pos: V, mass: V::Scalar },   // A localized gravity well/planet
}

impl<V: Vector> GravityModel<V> { 

    pub fn get(&self) -> V {
        match self {
            Self::Constant(g) => *g,
            Self::Zero => V::ZERO,
        }
    }
}

pub struct ParticleEnvironment<V, F> 
where 
    V: Vector,  
    F: CollisionFlags,
{       
    pub space: GridSpace<V>,
    pub tuning: SimulationTuning<V, F>,  
    pub state: State<V>,
    pub gravity: GravityModel<V>,
}

impl<V, F> ParticleEnvironment<V, F> 
where 
    V: Vector, 
    F: CollisionFlags,
{
    /// Creates a new parameterized particle environment.
    /// The static collision flag behavior is automatically inferred from the provided tuning configuration.
    #[inline(always)]
    pub fn new( 
        space: GridSpace<V>, 
        tuning: SimulationTuning<V, F>, 
        state: State<V>, 
        gravity: GravityModel<V>,
    ) -> Self {
        Self { 
            space,  
            tuning, 
            state,
            gravity,
        }
    }

   
}
impl<V,F> SubstepProvider for ParticleEnvironment<V,F> 
where 
    V: Vector, 
    F: CollisionFlags,
{
    #[inline]
    fn substep_count(&self) -> u64 { 
        self.tuning.substep_count
    }
}