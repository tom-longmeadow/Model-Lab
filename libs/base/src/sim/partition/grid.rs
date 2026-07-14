use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{AddAssign, Mul, Sub, SubAssign};
use crate::math::{Bounds, IVec2, IVec3, Vec2, Vec3, Vector};
use crate::sim::partition::Partition;

pub type UniformGrid2D = UniformGrid<IVec2, Vec2>;
pub type UniformGrid3D = UniformGrid<IVec3, Vec3>;

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

pub struct Collision<V> {
    pub a_index: usize,
    pub b_index: usize,
    pub normal: V,        
    pub penetration: f32,
}

impl<V> Collision<V> {
    pub fn new(mut a: usize, mut b: usize, normal: V, penetration: f32) -> Self {
        // Ensure deterministic ordering (a is always the smaller index)
        // Flip the normal if you swap the indices to maintain direction!
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        Self { a_index: a, b_index: b, normal, penetration }
    }
}

pub struct CollisionRegistry<V> {
    pub pairs: Vec<Collision<V>>, 
}

impl<V> CollisionRegistry<V> {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    pub fn push(&mut self, a_index: usize, b_index: usize, normal: V, penetration: f32) {
        self.pairs.push(Collision::new(a_index, b_index, normal, penetration));
    }

    /// Sorts collisions by penetration depth if you want to resolve deep penetrations first
    pub fn sort_by_depth(&mut self) {
        self.pairs.sort_unstable_by(|a, b| {
            b.penetration.partial_cmp(&a.penetration).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

pub struct UniformGrid<K, V> {  
    cell_size: f32,
    cells: HashMap<K, GridCell>, 
    _marker: std::marker::PhantomData<V>,
}
 
impl<K: Hash + Eq, V> UniformGrid<K, V> {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, cell_key: K, index: usize) {
        self.cells.entry(cell_key).or_default().indices.push(index);
    }

    #[inline]
    fn quantize(&self, value: f32) -> i32 {
        (value / self.cell_size).floor() as i32
    }
}

 impl<K: GridKey, V> UniformGrid<K, V> 
where
    V: Vector<Scalar = f32> + Sub<Output = V> + AddAssign + SubAssign + Mul<f32, Output = V>,
{
    #[inline(always)]
    fn check_and_append_collision(
        &self,
        idx_a: usize,
        idx_b: usize,
        positions: &[V],
        radii: &[f32], 
        collisions: &mut Vec<Collision<V>>
    ) {
        let delta = positions[idx_a] - positions[idx_b];
        let dist_sq = delta.dot(delta);
 
        let target_dist = radii[idx_a] + radii[idx_b];
        let target_dist_sq = target_dist * target_dist;

        if dist_sq < target_dist_sq && dist_sq > 0.0 {
            let distance = dist_sq.sqrt();
            let penetration = target_dist - distance;
            let normal = delta * (1.0 / distance); // Points from B to A

            collisions.push(Collision {
                a_index: idx_a,
                b_index: idx_b,
                normal,
                penetration,
            });
        }
    }
}

impl<K: GridKey, V> UniformGrid<K, V> 
where
    V: Vector<Scalar = f32> + Sub<Output = V> + AddAssign + SubAssign + Mul<f32, Output = V>,
{
    pub fn detect_all_collisions(
        &self, 
        positions: &[V], 
        radii: &[f32],  
        collisions: &mut Vec<Collision<V>>
    ) {
        for (cell_key, cell) in &self.cells {
            let indices = &cell.indices;
            let len = indices.len();

            // 1. INTRA-CELL
            if len >= 2 {
                for i in 0..len.saturating_sub(1) {
                    for j in (i + 1)..len {
                        self.check_and_append_collision(
                            indices[i], indices[j], positions, radii, collisions
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
                                idx_a, idx_b, positions, radii, collisions
                            );
                        }
                    }
                }
            }
        }
    }
}


impl UniformGrid<IVec2, Vec2> { 
    
    #[inline]
    pub fn quantize_position(&self, pos: Vec2) -> IVec2 {
        IVec2::new(
            self.quantize(pos.x),
            self.quantize(pos.y),
        )
    } 

    pub fn populate(&mut self, positions: &[Vec2]) {
        self.clear();
        for (index, &pos) in positions.iter().enumerate() {
            let cell_key = self.quantize_position(pos);
            self.insert(cell_key, index);
        }
    }
}
impl Partition<crate::math::Vec2> for crate::sim::partition::grid::UniformGrid2D {
    fn resize(&mut self, bounds: &Bounds, radii: &[f32]) {

        self.clear();
        if radii.is_empty() { return; }

        let mut max_radius: f32 = 0.0;
        for &r in radii { 
            max_radius = max_radius.max(r); 
        } 

        let largest_diameter = max_radius * 2.0;
        self.cell_size = largest_diameter * 1.1; 
    }

    fn clear(&mut self) {
        self.cells.clear();
    }

    fn populate(&mut self, positions: &[crate::math::Vec2]) { 
        self.populate(positions);
    }

    fn find_collisions(
        &self, 
        positions: &[crate::math::Vec2], 
        radii: &[f32], 
        registry: &mut CollisionRegistry<crate::math::Vec2>
    ) {
        self.detect_all_collisions(positions, radii, &mut registry.pairs);
    }
}



 


 