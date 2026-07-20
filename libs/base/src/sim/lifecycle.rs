 
use crate::sim::storage::{ElementStorage, Storage};
 
pub trait Lifecycle<S: Storage, Env> {
    fn tick(&mut self, storage: &mut S, tick: u64, dt: f64, environment: &Env);
}


pub trait ElementLifecycle<St: ElementStorage, Env> { 
    fn process_lifecycle(
        &mut self, 
        storage: &mut St, 
        tick: u64, 
        dt: f64,
        scratch_indices: &mut Vec<usize>, 
        environment: &Env
    );
}

impl<St, Env, LC> Lifecycle<St, Env> for LC
where
    St: ElementStorage + 'static,
    LC: ElementLifecycle<St, Env>,
{
    #[inline]
    fn tick(&mut self, storage: &mut St, tick: u64, dt: f64, environment: &Env) {
        // Allocate once inside your lifecycle container state to guarantee zero allocation overhead
        let mut scratch_indices = Vec::with_capacity(64);
        
        // Execute the simulation's custom structural evaluation
        self.process_lifecycle(storage, tick, dt, &mut scratch_indices, environment);
        
        // Flush all dead parent shells out of memory using your tail-swap algorithm
        storage.remove_indices(&mut scratch_indices);
    }
}


// /// The underlying buffers provided to the lifecycle runner.
// pub struct LifecycleItems<Item> {
//     pub spawn: Vec<Item>,
//     pub death: Vec<usize>,
// }

// /// A unified lifecycle trait that separates intent gathering from mutation.
// pub trait CpuLifecycleUnit<S: CpuStorage, Env> {
//     fn plan_lifecycle(
//         &mut self,
//         storage: &S,
//         environment: &Env,
//         tick: u64,
//         dt: f64,
//         buffers: &mut LifecycleItems<S::Item>,
//     );
// }

// /// The engine-facing implementation that drives execution.
// pub struct CpuLifecycleEngine<L, Item> {
//     pub unit: L,
//     buffers: LifecycleItems<Item>,
// }

// impl<L, Item> CpuLifecycleEngine<L, Item> {
//     pub fn new(unit: L) -> Self {
//         Self {
//             unit,
//             buffers: LifecycleItems {
//                 spawn: Vec::with_capacity(64),
//                 death: Vec::with_capacity(64),
//             },
//         }
//     }
// }

// impl<S, Env, L> Lifecycle<S, Env> for CpuLifecycleEngine<L, S::Item>
// where
//     S: CpuStorage,
//     L: CpuLifecycleUnit<S, Env>,

//     {
//     fn tick(&mut self, storage: &mut S, tick: u64, dt: f64, environment: &Env) {
//         // Step 1: Execute single unified lifecycle planning step
//         self.unit.plan_lifecycle(storage, environment, tick, dt, &mut self.buffers);

//         // Step 2: High-Performance Batch Execution
//         if !self.buffers.death.is_empty() {
//             storage.remove_indices(&mut self.buffers.death);
//             self.buffers.death.clear();
//         }

//         for item in self.buffers.spawn.drain(..) {
//             storage.push(item);
//         }
//     }
// }
// pub trait Creator<S: Storage, Env> {
//     type Item;
//     // Reads storage/env, writes new items to a buffer
//     fn plan_creations(&mut self, storage: &S, environment: &Env, output: &mut Vec<Self::Item>);
// }

// pub trait Deletor<S: Storage, Env> {
//     // Reads storage/env, writes indices targeted for deletion
//     fn plan_deletions(&mut self, storage: &S, environment: &Env, output: &mut Vec<usize>);
// }

 
// pub trait Policy<S: Storage, Env> {
//     fn try_create<C: Creator<S, Env>>(
//         &mut self, 
//         creator: &mut C, 
//         storage: &mut S, 
//         tick: u64, 
//         dt: f64, 
//         environment: &Env
//     );

//     fn try_delete<D: Deletor<S, Env>>(
//         &mut self, 
//         deletor: &mut D, 
//         storage: &mut S, 
//         tick: u64, 
//         dt: f64, 
//         environment: &Env
//     );
// }
 
// pub struct CpuLifecycle<C, D, P, Item> {
//     pub creator: C,
//     pub deletor: D,
//     pub policy: P,
//     // Recycled heap allocations to avoid allocations per frame
//     spawn_buffer: Vec<Item>,
//     death_buffer: Vec<usize>,
// }

// impl<S, Env, C, D, P> Lifecycle<S, Env> for CpuLifecycle<C, D, P, S::Item>
// where
//     S: CpuStorage, // Constrained to CpuStorage for batch mutations
//     C: Creator<S, Env, Item = S::Item>,
//     D: Deletor<S, Env>,
//     P: Policy<S, Env>, // Custom rules for thresholds/probabilites
// {
//     fn tick(&mut self, storage: &mut S, _tick: u64, _dt: f64, environment: &Env) {
//         // 1. Gather Intents (Read-only access to storage)
//         self.creator.plan_creations(storage, environment, &mut self.spawn_buffer);
//         self.deletor.plan_deletions(storage, environment, &mut self.death_buffer);

//         // 2. Policy Filtering (Optional curation step)
//         // e.g., self.policy.filter(&mut self.spawn_buffer, &mut self.death_buffer);

//         // 3. High-Performance Batch Execution
//         if !self.death_buffer.is_empty() {
//             storage.remove_indices(&mut self.death_buffer);
//             self.death_buffer.clear();
//         }

//         for item in self.spawn_buffer.drain(..) {
//             storage.push(item);
//         }
//     }
// }

// agnostic top level object that represents the creation and deletion policy of a simulation
//  pub trait Lifecycle<S: Storage, Env, C:Creator<S, Env>, D:Deletor<S,Env>> {   

    
//     fn tick(&mut self, storage: &mut S, tick: u64, step_dt: f64, _environment: &Env){
//         c.create
//     }
// }
//  pub struct Insert<Kind> {
//     pub kind: Kind,
//     pub count: usize,
// }

//  pub struct Kill<Kind> {
//     pub kind: Kind,
//     pub ids: Vec<usize>,
// }

// pub trait Creator<Env> {
//     /// The top-level heterogeneous item type that your storage natively holds.
//     type Output;

//     /// Builds a specific variant element based on the instructions inside the VariantKey.
//     fn create(
//         &mut self, 
//         key: Variant, 
//         index: usize, 
//         step_dt: f64, 
//         environment: &Env
//     ) -> Self::Output;
// }

// pub trait Deletor<S: Storage, Env> {
//     /// Evaluates whether a specific index holding a specific variant should be purged.
//     fn delete(
//         &mut self, 
//         key: Variant, 
//         index: usize, 
//         storage: &S, 
//         environment: &Env
//     ) -> bool;
// }

// pub trait Policy<S: Storage, Env, Kind> { 
//     fn evaluate_ingress(
//         &mut self, 
//         tick: u64, 
//         storage: &S, 
//         environment: &Env, 
//         create: &mut Vec<Insert<Kind>>,
//         delete: &mut Vec<Kill<Kind> >
//     );
// }




// pub trait IdentifiedStorage: Storage {
//     /// Returns the variant key for the item currently occupying this slot.
//     fn get_variant_key(&self, index: usize) -> Variant;
// }
// /// A zero-assumption builder for generating elements.
// pub trait Creator<Env> {
//     type Output;

//     /// Builds elements without making assumptions about layout or physics.
//     fn create(&mut self, index: usize, step_dt: f64, environment: &Env) -> Self::Output;
// }

// /// A zero-assumption evaluator for deciding when/how much to create.
// pub trait Policy<S: Storage, Env> {
//     /// Returns the precise quantity of elements to allocate on this tick.
//     ///fn evaluate_ingress(&mut self, tick: u64, storage: &S, environment: &Env) -> usize;
    

// }

// /// A zero-assumption filter for identifying dead elements.
// pub trait Deletor<S: Storage, Env> {
//     /// Evaluates if the element at `index` should be removed from the simulation.
//     fn delete(&mut self, index: usize, storage: &S, environment: &Env) -> bool;
// }

// pub struct StructuralLifecycle<Gov, Fac, Filt> {
//     pub governor: Gov,
//     pub factory: Fac,
//     pub filter: Filt,
//     // Reuse allocation buffers across frames to avoid heap thrashing
//     reclaim_buffer: Vec<usize>, 
// }

// impl<Gov, Fac, Filt> StructuralLifecycle<Gov, Fac, Filt> {
//     pub fn new(governor: Gov, factory: Fac, filter: Filt) -> Self {
//         Self {
//             governor,
//             factory,
//             filter,
//             reclaim_buffer: Vec::with_capacity(64),
//         }
//     }
// }

// impl<S, Env, Gov, Fac, Filt> Lifecycle<S, Env> for StructuralLifecycle<Gov, Fac, Filt>
// where
//     S: CpuStorage<Item = Fac::Output>,
//     Gov: Policy<S, Env>,
//     Fac: Creator<Env>,
//     Filt: Deletor<S, Env>,
// {
//     fn tick(&mut self, storage: &mut S, tick: u64, step_dt: f64, environment: &Env) {
//         // ==================================================================
//         // 1. THE DESTROY PHASE: Identify and purge dead elements
//         // ==================================================================
//         self.reclaim_buffer.clear();
//         let total_items = storage.len();

//         for idx in 0..total_items {
//             if self.filter.delete(idx, storage, environment) {
//                 self.reclaim_buffer.push(idx);
//             }
//         }

//         // Use your optimized, layout-agnostic batch-removal function
//         if !self.reclaim_buffer.is_empty() {
//             storage.remove_indices(&mut self.reclaim_buffer);
//         }

//         // ==================================================================
//         // 2. THE MAKE PHASE: Evaluate capacity constraints and allocate
//         // ==================================================================
//         let count_to_create = self.governor.evaluate_ingress(tick, storage, environment);
//         let max_headroom = storage.capacity().saturating_sub(storage.len());
//         let safe_spawn_count = count_to_create.min(max_headroom);

//         for i in 0..safe_spawn_count {
//             // Manufacture the item and push it directly into the storage container
//             let new_item = self.factory.create(i, step_dt, environment);
//             storage.push(new_item);
//         }
//     }
// }