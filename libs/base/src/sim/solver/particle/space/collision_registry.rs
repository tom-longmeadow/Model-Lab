
pub struct CollisionRegistry {
    pub a_indices: Vec<usize>,
    pub b_indices: Vec<usize>, 
}

impl Default for CollisionRegistry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl CollisionRegistry {
    pub fn new() -> Self {
        Self {
            a_indices: Vec::new(),
            b_indices: Vec::new(), 
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            a_indices: Vec::with_capacity(capacity),
            b_indices: Vec::with_capacity(capacity), 
        }
    }

    pub fn clear(&mut self) {
        self.a_indices.clear();
        self.b_indices.clear(); 
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.a_indices.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.a_indices.is_empty()
    }

    #[inline]
    pub fn push(
        &mut self,
        mut a: usize,
        mut b: usize, 
    ) {
        // Maintain a strict a < b order invariant for safe parallel/unchecked mutations
        if a > b {
            std::mem::swap(&mut a, &mut b); 
        }

        self.a_indices.push(a);
        self.b_indices.push(b); 
    }
}
  

