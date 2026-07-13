use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{AddAssign, Mul, Sub, SubAssign};
use crate::math::{DVec2, DVec3, FloatScalar, IVec2, IVec3, Vector};
use crate::sim::solver::partition::collision::{CollisionRegistry};

pub type UniformGrid2D = UniformGrid<IVec2, DVec2>;
pub type UniformGrid3D = UniformGrid<IVec3, DVec3>;

pub trait GridKey: Hash + Eq + Copy + std::ops::Add<Output = Self> + 'static { 
    const OFFSETS: &'static [Self];
}

impl GridKey for IVec2 {
    const OFFSETS: &'static [Self] = &[
        IVec2::new(1, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, 1),
        IVec2::new(1, 1),
    ];
}

impl GridKey for IVec3 {
    // 13 offsets representing exactly half of a 3x3x3 cube neighborhood.
    // This ensures no 3D cell pair is ever evaluated backward or duplicated.
    const OFFSETS: &'static [Self] = &[
        // Z = 0 plane (current layer, same as 2D)
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 1, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(1, 1, 0),
        
        // Z = 1 plane (the layer directly above/forward)
        IVec3::new(-1, -1, 1),
        IVec3::new(0, -1, 1),
        IVec3::new(1, -1, 1),
        IVec3::new(-1, 0, 1),
        IVec3::new(0, 0, 1),
        IVec3::new(1, 0, 1),
        IVec3::new(-1, 1, 1),
        IVec3::new(0, 1, 1),
        IVec3::new(1, 1, 1),
    ];
}

// --- DATA STRUCTURES ---

#[derive(Default)]
pub struct GridCell {
    pub indices: Vec<usize>,
}

impl GridCell { 
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct UniformGrid<K, V> {  
    cell_size: f64,
    cells: HashMap<K, GridCell>, 
    _marker: std::marker::PhantomData<V>,
}
 
impl<K: Hash + Eq, V> UniformGrid<K, V> {
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn set_cell_size(&mut self, cell_size: f64){
        self.cell_size = cell_size * 1.1;
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, cell_key: K, index: usize) {
        self.cells.entry(cell_key).or_default().indices.push(index);
    }
 
}

impl<K: GridKey, V> UniformGrid<K, V>
where
    V: Vector + Sub<Output = V> + AddAssign + SubAssign + Mul<V::Scalar, Output = V>,
    V::Scalar: FloatScalar,
{
   #[inline(always)]
    fn check_and_append_collision(
        &self,
        idx_a: usize,
        idx_b: usize,
        positions: &[V],
        radii: &[V::Scalar],
        registry: &mut CollisionRegistry<V>,
    ) {
        let delta = positions[idx_a] - positions[idx_b];
        let dist_sq = delta.dot(delta);
        let target_dist = radii[idx_a] + radii[idx_b];

        if dist_sq < target_dist * target_dist && dist_sq > V::Scalar::ZERO {
            let distance = dist_sq.sqrt();
            let normal = delta * (V::Scalar::ONE / distance);
            registry.push(idx_a, idx_b, normal, target_dist - distance);
        }
    }

   pub fn find_collisions(
        &self,
        positions: &[V],
        radii: &[V::Scalar],
        registry: &mut CollisionRegistry<V>,
    ) {
        for (cell_key, cell) in &self.cells {
            let indices = &cell.indices;
            let len = indices.len();

            // 1. INTRA-CELL
            if len >= 2 {
                for i in 0..len.saturating_sub(1) {
                    for j in (i + 1)..len {
                        self.check_and_append_collision(
                            indices[i], indices[j], positions, radii, registry,
                        );
                    }
                }
            }

            // 2. NEIGHBOR-CELL
            for &offset in K::OFFSETS {
                let neighbor_key = *cell_key + offset;
                if let Some(neighbor_cell) = self.cells.get(&neighbor_key) {
                    for &idx_a in indices {
                        for &idx_b in &neighbor_cell.indices {
                            self.check_and_append_collision(
                                idx_a, idx_b, positions, radii, registry,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub trait QuantizeInto<K> {
    fn quantize_into(self, cell_size: f64) -> K;
}

impl QuantizeInto<IVec2> for DVec2 {
    fn quantize_into(self, cell_size: f64) -> IVec2 {
        IVec2::new(
            (self.x / cell_size).floor() as i32,
            (self.y / cell_size).floor() as i32,
        )
    }
}

impl QuantizeInto<IVec3> for DVec3 {
    fn quantize_into(self, cell_size: f64) -> IVec3 {
        IVec3::new(
            (self.x / cell_size).floor() as i32,
            (self.y / cell_size).floor() as i32,
            (self.z / cell_size).floor() as i32,
        )
    }
}

 
impl<K: GridKey, V> UniformGrid<K, V>
where
    V: Copy + QuantizeInto<K>,
{
    pub fn populate(&mut self, positions: &[V]) {
        self.clear();
        for (index, &pos) in positions.iter().enumerate() {
            let cell_key = pos.quantize_into(self.cell_size);
            self.insert(cell_key, index);
        }
    }
}
 
 



 


 