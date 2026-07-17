use std::collections::HashMap;
use std::hash::Hash; 
use crate::{math::{FloatScalar,  Vector}, sim::solver::particle::{space::{collision_registry::CollisionRegistry, grid_key::GridKey}, verlet_particle::VerletParticle, verlet_soa_vec_storage::ComponentSliceMut}}; 
 
 

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
    pub cells: HashMap<V::Quantized, GridCell>, 
    _marker: std::marker::PhantomData<V>,
}

impl<V: Vector> UniformGrid<V> 
where
    V::Quantized: Hash + Eq,
{
    pub fn new(cell_size: V::Scalar) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn set_cell_size(&mut self, cell_size: V::Scalar) { 
        let buffer_factor = <V::Scalar as FloatScalar>::from_f64(1.1);
        self.cell_size = cell_size * buffer_factor;
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, cell_key: V::Quantized, index: usize) {
        self.cells.entry(cell_key).or_default().indices.push(index);
    }

    pub fn populate(&mut self, positions: &[V]) {
        self.clear();
        for (index, &pos) in positions.iter().enumerate() {
            // position calls your macro trait directly into its internal associated match
            let cell_key = pos.quantize_into(self.cell_size);
            self.insert(cell_key, index);
        }
    }
}

impl<V: Vector> UniformGrid<V> 
where 
    <V as Vector>::Quantized: Hash + Eq + Copy,
{
     #[inline(always)]
    fn traverse_grid_cells(&self, mut check_pair: impl FnMut(usize, usize)) {
        type QKey<VecT> = <VecT as Vector>::Quantized;

        for (cell_key, cell) in &self.cells {
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
            for &offset in QKey::<V>::OFFSETS { 
                let neighbor_key = *cell_key + offset;
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

    pub fn soa_find_collisions(
        &self,
        pos_x: &ComponentSliceMut<V::Scalar>,
        pos_y:&ComponentSliceMut<V::Scalar>,
        radii: &[V::Scalar], // 🟢 FIXED: Remapped to standard flat slice layout
        registry: &mut CollisionRegistry,
    ) {
        self.traverse_grid_cells(|a, b| {
            self.soa_check_for_collision(a, b, pos_x, pos_y, radii, registry);
        });
    }

    pub fn aos_find_collisions(
        &self,
        particles: &[VerletParticle<V>], 
        registry: &mut CollisionRegistry,
    ) {
        self.traverse_grid_cells(|a, b| {
            self.aos_check_for_collision(a, b, particles, registry);
        });
    }

    #[inline(always)]
    fn soa_check_for_collision(
        &self,
        idx_a: usize,
        idx_b: usize,
        pos_x: &ComponentSliceMut<V::Scalar>,   
        pos_y: &ComponentSliceMut<V::Scalar>,
        radii: &[V::Scalar], // 🟢 FIXED: Remapped to standard flat slice layout
        registry: &mut CollisionRegistry,
    ) {
        // 🟢 FIXED: Replaced standard array indexing with your optimized strided getters
        unsafe {
            let dx = pos_x.get_unchecked(idx_b) - pos_x.get_unchecked(idx_a); 
            let dy = pos_y.get_unchecked(idx_b) - pos_y.get_unchecked(idx_a);
            let dist_sq = dx * dx + dy * dy;
            let target_dist = radii[idx_a] + radii[idx_b];

            if dist_sq < target_dist * target_dist || dist_sq == V::Scalar::ZERO {
                registry.push(idx_a, idx_b);
            }
        }
    }
 
    #[inline(always)]
    fn aos_check_for_collision(
        &self,
        idx_a: usize,
        idx_b: usize,
        particles: &[VerletParticle<V>],  
        registry: &mut CollisionRegistry,
    ) {
        let p_a = &particles[idx_a];
        let p_b = &particles[idx_b];

        let delta = p_b.pos - p_a.pos; 
        let dist_sq = delta.length_squared();
        let target_dist = p_a.radius + p_b.radius;

        // 🟢 FIXED: Fused particles (dist_sq == 0) are no longer dropped!
        if dist_sq < target_dist * target_dist || dist_sq == V::Scalar::ZERO {
            registry.push(idx_a, idx_b);
        }
    }
}

// impl<V: Vector> UniformGrid<V>
// where
//     V::Quantized: GridKey + Hash + Eq,
// {

    
    
    // #[inline(always)]
    // fn check_and_append_collision(
    //     &self,
    //     idx_a: usize,
    //     idx_b: usize,
    //     positions: &[V],
    //     radii: &[V::Scalar],
    //     registry: &mut CollisionRegistry<V>,
    // ) {
    //     let delta = positions[idx_a] - positions[idx_b];
    //     let dist_sq = delta.dot(delta);
    //     let target_dist = radii[idx_a] + radii[idx_b];

    //     if dist_sq < target_dist * target_dist && dist_sq > V::Scalar::ZERO {
    //         let distance = dist_sq.sqrt();
    //         let normal = delta * (V::Scalar::ONE / distance);
    //         registry.push(idx_a, idx_b, normal, target_dist - distance);
    //     }
    // }

    // pub fn find_collisions(
    //     &self,
    //     positions: &[V],
    //     radii: &[V::Scalar],
    //     registry: &mut CollisionRegistry<V>,
    // ) {
    //     // Grab the internal offsets defined by your quantized type's trait
    //     type QKey<VecT> = <VecT as Vector>::Quantized;

    //     for (cell_key, cell) in &self.cells {
    //         let indices = &cell.indices;
    //         let len = indices.len();

    //         // 1. INTRA-CELL (Self-collisions within the same grid unit)
    //         if len >= 2 {
    //             for i in 0..len.saturating_sub(1) {
    //                 for j in (i + 1)..len {
    //                     self.check_and_append_collision(
    //                         indices[i], indices[j], positions, radii, registry,
    //                     );
    //                 }
    //             }
    //         }

    //         // 2. NEIGHBOR-CELL (Querying surrounding boundaries)
    //         for &offset in QKey::<V>::OFFSETS {
    //             let neighbor_key = *cell_key + offset;
    //             if let Some(neighbor_cell) = self.cells.get(&neighbor_key) {
    //                 for &idx_a in indices {
    //                     for &idx_b in &neighbor_cell.indices {
    //                         self.check_and_append_collision(
    //                             idx_a, idx_b, positions, radii, registry,
    //                         );
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
// }
 

 
 

 
 



 


 