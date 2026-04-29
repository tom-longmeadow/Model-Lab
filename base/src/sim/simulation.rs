use crate::sim::{clock::Clock, solver::Solver, storage::Storage};

pub struct Simulation<St, Sv>
where
    St: Storage,
    Sv: Solver<St>,
{
    storage: St,
    solver:  Sv,
    clock:   Clock,
}
 
/// Interface for an SOA simulation.
impl<St, Sv> Simulation<St, Sv>
where
    St: Storage,
    Sv: Solver<St>,
{
    pub fn new(hz: f64, storage: St, solver: Sv) -> Self {
        Self {
            storage,
            solver,
            clock: Clock::new(hz),
        }
    }

    pub fn clock(&self) -> &Clock { &self.clock }

    pub fn storage(&self) -> &St { &self.storage }

    pub fn simulate(&mut self, frame_time: f64) {
        let tick  = self.clock.tick();
        let steps = self.clock.advance(frame_time);
        let subs  = self.solver.substep_count().max(1);
        let dt    = self.clock.fixed_dt() / subs as f64;

        for step in 0..steps {
            let current_tick = tick + step as u64;

            self.storage.pre_step();
            self.solver.pre_step(&mut self.storage, self.clock.fixed_dt(), current_tick);

            for _ in 0..subs {
                self.solver.substep(&mut self.storage, dt);
            }

            self.solver.post_step(&mut self.storage);
            self.storage.post_step();
        }
    }
}




/********************/ 
/*      TESTS       */ 
/********************/ 
#[cfg(test)]
mod tests {  
    use super::*;   

   pub struct MockEntity {
        pub d64: f64,
        pub c8:  u8,
    }

    pub struct MockStorage {
        data: Vec<MockEntity>,
    }

    impl Storage for MockStorage {
        type Item = MockEntity;

        fn new(capacity: usize) -> Self {
            Self { data: Vec::with_capacity(capacity) }
        }

        fn len(&self)      -> usize { self.data.len() }
        fn capacity(&self) -> usize { self.data.capacity() }
        fn read(&self)     -> &[MockEntity] { &self.data }
        fn write(&mut self) -> &mut [MockEntity] { &mut self.data }
    }

    impl MockStorage {
        // tests push entities via this helper
        pub fn push(&mut self, item: MockEntity) { self.data.push(item); }
        pub fn get(&self, index: usize) -> &MockEntity { &self.data[index] }
    }
    pub struct MockSolver {
        pub calls: String,
        pub received_ticks: Vec<u64>,
        pub received_dts: Vec<f64>,
        pub iteration_count: usize,
    }

    impl MockSolver {
        pub fn new() -> Self {
            Self {
                calls: String::new(), 
                received_ticks: Vec::new(),
                received_dts: Vec::new(),
                iteration_count: 2,
            }
        }
    }
 
    impl Solver<MockStorage> for MockSolver {


        fn substep_count(&self) -> usize { self.iteration_count }

         
 
        fn pre_step(&mut self, storage: &mut MockStorage, _: f64, tick: u64) {
            self.calls.push_str("prepare-");
            self.received_ticks.push(tick);
            storage.push(setup_entity1());
        }
  
        fn substep(&mut self, _: &mut MockStorage, dt: f64){
            self.calls.push_str("sub-"); 
            self.received_dts.push(dt);
        }


        fn post_step(&mut self, _: &mut MockStorage){
            self.calls.push_str("finalize"); 
        } 
    }
      
   
    fn setup_sim() -> Simulation<MockStorage, MockSolver> {
        let solver = MockSolver::new();
        let storage = MockStorage::new(10);
        let sim = Simulation::new(100.0, storage, solver);
        return sim;
    }

    fn setup_entity1() -> MockEntity {
        MockEntity { d64: 1.2, c8: 5 }
    }

     #[test]
    fn test_initialization() {
        let sim = setup_sim();  
        assert_eq!(sim.storage.capacity(), 10);
        assert_eq!(sim.clock().elapsed_time(), 0.0);
        assert_eq!(sim.clock().fixed_dt(), 0.01);
        assert_eq!(sim.clock().tick(), 0); 
    }

   
     #[test]
    fn test_sim_step_execution() { 
        let mut sim = setup_sim(); 
        let e = setup_entity1(); 

        sim.simulate(0.01);
 
        assert_eq!(sim.solver.calls, "prepare-sub-sub-finalize"); 
        assert_eq!(sim.storage.capacity(), 10);
        assert_eq!(sim.storage.len(), 1);
        assert_eq!(sim.storage.get(0).d64, e.d64);
        assert_eq!(sim.storage.get(0).c8,  e.c8);
    }

   

    #[test]
    fn test_tick_sequence() {
        let mut sim = setup_sim();   
        let dt = sim.clock().fixed_dt();

        sim.simulate(dt); 
        assert_eq!(sim.clock().tick(), 1);
        assert_eq!(sim.clock().elapsed_time(), dt);
         
        sim.simulate(dt * 2.0); 
        assert_eq!(sim.clock().tick(), 3);
        assert_eq!(sim.clock().elapsed_time(), dt * 3.0);
    }


 
    #[test] 
    fn test_zero_iterations_safety() {
        let mut sim = setup_sim();  
        let dt = sim.clock().fixed_dt();


        sim.simulate(0.0); 
        assert_eq!(sim.clock().tick(), 0);
        assert_eq!(sim.clock().elapsed_time(),0.0);

        sim.solver.iteration_count = 0;
        sim.simulate(dt); 

        assert_eq!(sim.clock().tick(), 1);
        assert_eq!(sim.clock().elapsed_time(), dt);
    }


    #[test]
    fn test_tick_increment_across_multiple_steps() {
        let mut sim = setup_sim();   
        let dt = sim.clock().fixed_dt();

        sim.simulate(dt * 2.0); 
        assert_eq!(sim.solver.received_ticks, vec![0, 1]);
        
         
        sim.simulate(dt * 1.5);   
        assert_eq!(sim.solver.received_ticks, vec![0, 1, 2]);

        sim.simulate(dt * 0.5);  
        assert_eq!(sim.solver.received_ticks, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_substep_dt_scaling() {
        let mut sim = setup_sim();   
        let count = sim.solver.substep_count();
        let dt_step = sim.clock().fixed_dt();
        let dt_substep = dt_step / (count as f64);

        sim.simulate(dt_step * 4.0);
         
        assert_eq!(sim.solver.received_dts, vec![dt_substep; count * 4] );  
    }


    #[test]
    fn test_storage_mutation_count() {
        let mut sim = setup_sim();  
        let dt_step = sim.clock().fixed_dt(); 
        sim.simulate(dt_step * 2.1); 
        assert_eq!(sim.storage.len(), 2);
    }

    

}
