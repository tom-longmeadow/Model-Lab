
use crate::math::Vector;

pub struct Collision<V: Vector> {
    pub a_index: usize,
    pub b_index: usize,
    pub normal: V,
    pub penetration: V::Scalar,
}

impl<V: Vector> Collision<V> {
    pub fn new(mut a: usize, mut b: usize, normal: V, penetration: V::Scalar) -> Self {
        // Ensure deterministic ordering (a is always the smaller index)
        // Flip the normal if you swap the indices to maintain direction!
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        Self { a_index: a, b_index: b, normal, penetration }
    }
}

pub struct CollisionRegistry<V: Vector> {
    pub pairs: Vec<Collision<V>>,
}

impl<V: Vector> Default for CollisionRegistry<V> {
    fn default() -> Self { Self::new() }
}

impl<V: Vector> CollisionRegistry<V> {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    pub fn push(&mut self, a_index: usize, b_index: usize, normal: V, penetration: V::Scalar) {
        self.pairs.push(Collision::new(a_index, b_index, normal, penetration));
    }

    /// Sorts collisions by penetration depth if you want to resolve deep penetrations first
    pub fn sort_by_depth(&mut self)
    where
        V::Scalar: PartialOrd,
    {
        self.pairs.sort_unstable_by(|a, b| {
            b.penetration.partial_cmp(&a.penetration).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

