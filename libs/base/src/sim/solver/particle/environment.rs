use crate::math::Vector;
use crate::prelude::solver::particle::space::GridSpace;
use crate::prelude::solver::particle::tuning::SimulationTuning;
use crate::prelude::solver::particle::runtime::RuntimeState;
use crate::sim::simulation::SubstepProvider;


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

pub struct ParticleEnvironment<V> 
where 
    V: Vector 
{
    pub space: GridSpace<V>,
    pub tuning: SimulationTuning<V>,
    pub state: RuntimeState<V>,
    pub gravity: GravityModel<V>,
}

impl<V: Vector> ParticleEnvironment<V> {
    pub fn new(space: GridSpace<V>, tuning: SimulationTuning<V>, state: RuntimeState<V>, 
        gravity: GravityModel<V>) -> Self {
        Self {
            space,  
            tuning, 
            state,
            gravity
        }
    }
}

impl<V: Vector> SubstepProvider for ParticleEnvironment<V> {
    #[inline]
    fn substep_count(&self) -> u64 { 
        self.tuning.substep_count
    }
}