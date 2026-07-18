 
use std::hash::Hash; 
use crate::{math::{FloatScalar,  Vector}, 
sim::solver::particle::{environment::ParticleEnvironment, space::{collision_registry::CollisionRegistry, grid_key::GridKey}, verlet_particle::VerletParticle}}; 
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
    active_keys: Vec<V::Quantized>, // 🏎️ Fast linear iterator cache
    _marker: std::marker::PhantomData<V>,
}

impl<V: Vector> UniformGrid<V> 
where
    V::Quantized: Hash + Eq + Copy,
{
    pub fn new(cell_size: V::Scalar) -> Self {
        Self {
            cell_size,
            cells: FxHashMap::default(),
            active_keys: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

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

    pub fn insert(&mut self, cell_key: V::Quantized, index: usize) {
        if let Some(cell) = self.cells.get_mut(&cell_key) {
            // Track key if this cell is transitioning from empty to active
            if cell.indices.is_empty() {
                self.active_keys.push(cell_key);
            }
            cell.indices.push(index);
        } else {
            let mut cell = GridCell::new();
            cell.indices.push(index);
            self.cells.insert(cell_key, cell);
            self.active_keys.push(cell_key);
        }
    }

    pub fn populate(&mut self, positions: &[V]) {
        self.clear(); // O(Active Cells), zero runtime heap allocations!
        for (index, &pos) in positions.iter().enumerate() {
            let cell_key = pos.quantize_into(self.cell_size);
            self.insert(cell_key, index);
        }
    }
}

impl<V: Vector> UniformGrid<V> 
where 
    <V as Vector>::Quantized: Hash + Eq + Copy,
{
    /// Resolves particle collisions directly across the spatial grid layout.
    /// Safely applies time-isolated restitution impulses alongside geometric position constraints.
   pub unsafe fn soa_resolve_collisions_spatial_direct(
        &self,
        positions: &mut [V],
        positions_old: &mut [V], // 🟢 ENSURE THIS IS UNPACKED AND PASSED IN
        inv_masses: &[V::Scalar],
        radii: &[V::Scalar],
        v_dt: V::Scalar, 
        env: &ParticleEnvironment<V>,
    ) {
        type QKey<VecT> = <VecT as Vector>::Quantized;

        let p_ptr = positions.as_mut_ptr();
        let p_old_ptr = positions_old.as_mut_ptr(); 
        let m_ptr = inv_masses.as_ptr();
        let r_ptr = radii.as_ptr();

        let slop = env.tuning.physics.penetration_slop;
        let bias = env.tuning.physics.penetration_correction_bias;
        let base_jitter = env.state.runtime_jitter;
        let restitution = env.tuning.physics.restitution;

        let iterations_count = env.tuning.collision_iterations;
        if iterations_count == 0 { return; }

        // 🟢 CRITICAL TIME SCALING: Calculate dt per individual iteration pass
        // Prevents multiple loop passes from multiplying the impulse velocity artificially
        let iter_dt = v_dt / <V::Scalar as FloatScalar>::from_f64(iterations_count as f64);

        // 🏎️ Outer iteration loop keeps active particles warm in L1/L2 cache
        for _ in 0..iterations_count {
            
            // ⚡ LINEAR SCAN: Perfectly contiguous memory iteration over active spatial cells
            for &cell_key in &self.active_keys {
                // Safe to unwrap as active_keys only mirrors valid entries
                let cell = self.cells.get(&cell_key).unwrap_unchecked();
                let indices = &cell.indices;
                let len = indices.len();

                // Helper macro to inline the raw mathematical resolution step
               macro_rules! resolve_pair {
    ($idx_a:expr, $idx_b:expr) => {{
        let a = *$idx_a;
        let b = *$idx_b;

        let pos_a_ptr = p_ptr.add(a);
        let pos_b_ptr = p_ptr.add(b);

        let mut delta = *pos_a_ptr - *pos_b_ptr;
        let mut dist_sq = delta.length_squared();
        let target_dist = *r_ptr.add(a) + *r_ptr.add(b);

        if dist_sq == V::Scalar::ZERO {
            let sep_arr = [<V::Scalar as FloatScalar>::from_f64(0.0001), V::Scalar::ZERO];
            delta = V::from_slice(&sep_arr);
            dist_sq = delta.length_squared();
        }

        if dist_sq < target_dist * target_dist {
            let dist = dist_sq.sqrt();
            let raw_penetration = target_dist - dist;

            if raw_penetration > slop {
                let penetration = raw_penetration - slop;
                let mut normal = delta / dist;

                let jitter_vec = normal.mul_elementwise(base_jitter);
                normal = normal + jitter_vec;
                let normal_len_sq = normal.length_squared();
                if normal_len_sq > V::Scalar::ZERO {
                    normal = normal / normal_len_sq.sqrt();
                }

                let inv_mass_a = *m_ptr.add(a);
                let inv_mass_b = *m_ptr.add(b);
                let total_inv_mass = inv_mass_a + inv_mass_b;

                if total_inv_mass > V::Scalar::ZERO {
                    // 1. PURE GEOMETRIC RESOLUTION (Your original stable code)
                    let response_magnitude = (penetration * bias) / total_inv_mass;
                    let displacement_a = normal * (response_magnitude * inv_mass_a);
                    let displacement_b = normal * (response_magnitude * inv_mass_b);

                    // Apply the precise position shifts that kept your simulation stable
                    *pos_a_ptr = *pos_a_ptr + displacement_a;
                    *pos_b_ptr = *pos_b_ptr - displacement_b;

                    // 2. SAFE VELOCITY RESTITUTION (Alters velocity for the NEXT substep)
                    if restitution > V::Scalar::ZERO {
                        let p_old_a_ptr = p_old_ptr.add(a);
                        let p_old_b_ptr = p_old_ptr.add(b);

                        // Use your substep v_dt (not iter_dt) because true velocity spans the whole substep
                        let vel_a = (*pos_a_ptr - *p_old_a_ptr) / v_dt;
                        let vel_b = (*pos_b_ptr - *p_old_b_ptr) / v_dt;
                        let relative_velocity = vel_a - vel_b;
                        let vel_along_normal = relative_velocity.dot(normal);

                        // Only bounce if they are moving towards each other relative to the substep timeline
                        if vel_along_normal < V::Scalar::ZERO {
                            let impulse_scalar = -(V::Scalar::ONE + restitution) * vel_along_normal;
                            let impulse_magnitude = impulse_scalar / total_inv_mass;
                            let impulse_vec = normal * impulse_magnitude;

                            // Turn the velocity impulse into a history shift vector
                            let history_shift = impulse_vec * v_dt;

                            // Update pos_old directly. This does not disrupt the current loops' positions,
                            // ensuring your 0.4 bias can stack and settle beautifully!
                            *p_old_a_ptr = *p_old_a_ptr - (history_shift * inv_mass_a);
                            *p_old_b_ptr = *p_old_b_ptr + (history_shift * inv_mass_b);
                        }
                    }
                }
            }
        }
    }};
}

                // 1. INTRA-CELL (Self-collisions within the same grid container cell)
                if len >= 2 {
                    for i in 0..len.saturating_sub(1) {
                        for j in (i + 1)..len {
                            resolve_pair!(indices.get_unchecked(i), indices.get_unchecked(j));
                        }
                    }
                }

                // 2. NEIGHBOR-CELL (Querying adjacent localized grid cell buckets)
                for &offset in QKey::<V>::OFFSETS { 
                    let neighbor_key = cell_key + offset;
                    if let Some(neighbor_cell) = self.cells.get(&neighbor_key) {
                        for i in 0..len {
                            for j in 0..neighbor_cell.indices.len() {
                                resolve_pair!(
                                    indices.get_unchecked(i), 
                                    neighbor_cell.indices.get_unchecked(j)
                                );
                            }
                        }
                    }
                }
            }
        }
    }
 


    #[inline(always)]
    fn traverse_grid_cells(&self, mut check_pair: impl FnMut(usize, usize)) {
        type QKey<VecT> = <VecT as Vector>::Quantized;

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
            for &offset in QKey::<V>::OFFSETS { 
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

    // pub fn soa_find_collisions(
    //     &self,
    //     positions: &[V],       
    //     radii: &[V::Scalar],   
    //     registry: &mut CollisionRegistry,
    // ) {
    //     self.traverse_grid_cells(|a, b| {
    //         self.soa_check_for_collision(a, b, positions, radii, registry);
    //     });
    // }

    // #[inline(always)]
    // fn soa_check_for_collision(
    //     &self,
    //     idx_a: usize,
    //     idx_b: usize,
    //     positions: &[V],
    //     radii: &[V::Scalar],
    //     registry: &mut CollisionRegistry,
    // ) {
    //     unsafe {
    //         let p_a = *positions.get_unchecked(idx_a);
    //         let p_b = *positions.get_unchecked(idx_b);

    //         let delta = p_b - p_a; 
    //         let dist_sq = delta.length_squared();
    //         let target_dist = *radii.get_unchecked(idx_a) + *radii.get_unchecked(idx_b);

    //         if dist_sq <= target_dist * target_dist {
    //             registry.push(idx_a, idx_b);
    //         }
    //     }
    // }

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
    fn aos_check_for_collision(
        &self,
        idx_a: usize,
        idx_b: usize,
        particles: &[VerletParticle<V>],
        registry: &mut CollisionRegistry,
    ) {
        unsafe {
            let p_a = particles.get_unchecked(idx_a);
            let p_b = particles.get_unchecked(idx_b);

            let delta = p_b.pos - p_a.pos; 
            let dist_sq = delta.length_squared();
            let target_dist = p_a.radius + p_b.radius;

            if dist_sq <= target_dist * target_dist {
                registry.push(idx_a, idx_b);
            }
        }
    }
}