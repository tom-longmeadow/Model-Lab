 
use std::hash::Hash; 
use crate::{math::{FloatScalar,  Vector}, 
sim::solver::particle::{space::{ grid_key::GridKey}}}; 
 use rustc_hash::FxHashMap;
 

#[derive(Default)]
pub struct GridCell {
    pub indices: Vec<usize>,
}

impl GridCell { 
    pub fn new() -> Self {
        Self::default()
    }
}
pub struct UniformGrid<V> 
where
    V: Vector,
{  
    pub cell_size: V::Scalar,   
    pub cells: FxHashMap<V::Quantized, GridCell>, 
    pub active_keys: Vec<V::Quantized>,  
    _marker: std::marker::PhantomData<V>,
}

impl<V: Vector> UniformGrid<V> 
where
    V::Quantized: Hash + Eq + Copy + GridKey
{
    pub fn new(cell_size: V::Scalar) -> Self {
        Self {
            cell_size,
            cells: FxHashMap::default(),
            active_keys: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn set_cell_size(&mut self, cell_size: V::Scalar) { 
        let buffer_factor = <V::Scalar as FloatScalar>::from_f64(1.1);
        self.cell_size = cell_size * buffer_factor;
    }

    /// Retains internal memory allocations to completely eliminate runtime heap churn
    pub fn clear(&mut self) {
        // ⚡ OPTIMIZATION: Only clear active cells, avoiding full sparse map iterations
        for key in &self.active_keys {
            if let Some(cell) = self.cells.get_mut(key) {
                cell.indices.clear();
            }
        }
        self.active_keys.clear();
    }

    /// Optional: Call this when changing scenes or levels to prevent empty cell accumulation leaks
    pub fn shrink_to_fit(&mut self) {
        self.cells.retain(|_, cell| !cell.indices.is_empty());
        self.cells.shrink_to_fit();
        self.active_keys.shrink_to_fit();
    }

    #[inline(always)]
    pub fn insert(&mut self, cell_key: V::Quantized, index: usize) {
        // Use the Entry API to perform exactly ONE hash lookup per particle
        match self.cells.entry(cell_key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let cell = entry.get_mut();
                // If it was cleared earlier this frame, it's now active again
                if cell.indices.is_empty() {
                    self.active_keys.push(cell_key);
                }
                cell.indices.push(index);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                // Completely new cell allocation (rare after initial frames)
                let mut cell = GridCell::new();
                cell.indices.push(index);
                entry.insert(cell);
                self.active_keys.push(cell_key);
            }
        }
    }

    pub fn populate(&mut self, positions: &[V]) {
        self.clear(); 
        
        // Cache the cell size in a local variable to prevent repeating pointer dereferences in the loop
        let cached_size = self.cell_size; 
        
        for (index, pos) in positions.iter().enumerate() {
            let cell_key = pos.quantize_into(cached_size);
            self.insert(cell_key, index);
        }
    }

    pub fn populate_sort(&mut self, positions: &[V]) 
    where
       V::Quantized: Hash + Eq + Copy + GridKey,
    {
        Self::populate(self, positions); 
    
        // ⚡ HIGH-PERFORMANCE CACHE OPTIMIZATION: 
        // Compiles down to direct register/assembly comparisons of the primitive integers
        self.active_keys.sort_unstable_by_key(|key| key.to_array_layout());
    }

}

impl<V: Vector> UniformGrid<V> 
where 
    V::Quantized: Hash + Eq + Copy + GridKey
{
   

     #[inline(always)]
    pub fn traverse_grid_cells(&self, mut check_pair: impl FnMut(usize, usize)) {
        // ⚡ OPTIMIZATION: Perfectly linear memory scan over contiguous active keys
        for &cell_key in &self.active_keys {
            // Safe to unwrap or get unchecked as active_keys only contains existing entries
            let cell = unsafe { self.cells.get(&cell_key).unwrap_unchecked() };
            let indices = &cell.indices;
            let len = indices.len();

            // 1. INTRA-CELL (Self-collisions within the same grid container cell)
            if len >= 2 {
                for i in 0..len.saturating_sub(1) {
                    for j in (i + 1)..len {
                        check_pair(indices[i], indices[j]);
                    }
                }
            }

            // 2. NEIGHBOR-CELL (Querying adjacent localized grid cell buckets)
            // Use the associated type directly via the bound trait
            for &offset in V::Quantized::OFFSETS { 
                let neighbor_key = cell_key + offset;
                if let Some(neighbor_cell) = self.cells.get(&neighbor_key) {
                    for &idx_a in indices {
                        for &idx_b in &neighbor_cell.indices {
                            check_pair(idx_a, idx_b);
                        }
                    }
                }
            }
        }
    }

     
    // pub fn aos_find_collisions(
    //     &self,
    //     particles: &[VerletParticle<V>], 
    //     registry: &mut CollisionRegistry,
    // ) {
    //     self.traverse_grid_cells(|a, b| {
    //         self.aos_check_for_collision(a, b, particles, registry);
    //     });
    // }

    // #[inline(always)]
    // fn aos_check_for_collision(
    //     &self,
    //     idx_a: usize,
    //     idx_b: usize,
    //     particles: &[VerletParticle<V>],
    //     registry: &mut CollisionRegistry,
    // ) {
    //     unsafe {
    //         let p_a = particles.get_unchecked(idx_a);
    //         let p_b = particles.get_unchecked(idx_b);

    //         let delta = p_b.pos - p_a.pos; 
    //         let dist_sq = delta.length_squared();
    //         let target_dist = p_a.radius + p_b.radius;

    //         if dist_sq <= target_dist * target_dist {
    //             registry.push(idx_a, idx_b);
    //         }
    //     }
    // }
}