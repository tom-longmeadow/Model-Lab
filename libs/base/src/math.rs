use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, Div, Neg};


pub use glam::FloatExt;
pub const EPSILON: f32 = 1e-5;

// ==========================================
// Your Public API Type Aliases
// ==========================================
pub type BVec2 = glam::BVec2;
pub type BVec3 = glam::BVec3;
pub type BVec4 = glam::BVec4;

pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;

pub type DVec2 = glam::DVec2;
pub type DVec3 = glam::DVec3;
pub type DVec4 = glam::DVec4;

pub type IVec2 = glam::IVec2;
pub type IVec3 = glam::IVec3;
pub type IVec4 = glam::IVec4;

pub type UVec2 = glam::UVec2;
pub type UVec3 = glam::UVec3;
pub type UVec4 = glam::UVec4;

pub type Mat2 = glam::Mat2;
pub type Mat3 = glam::Mat3;
pub type Mat4 = glam::Mat4;

pub type DMat2 = glam::DMat2;
pub type DMat3 = glam::DMat3;
pub type DMat4 = glam::DMat4;

pub type Quat = glam::Quat;
pub type DQuat = glam::DQuat;

pub type Affine2 = glam::Affine2;
pub type Affine3A = glam::Affine3A;




pub trait QuantizeInto<K, S> {
    fn quantize_into(self, cell_size: S) -> K;
}
macro_rules! impl_quantize {
    ($from_type:ty, $to_type:ty, $scalar:ty, [$($field:ident),+]) => {
        impl QuantizeInto<$to_type, $scalar> for $from_type {
            #[inline]
            fn quantize_into(self, cell_size: $scalar) -> $to_type {
                <$to_type>::new(
                    $((self.$field / cell_size).floor() as i32),+
                )
            }
        }
    };
}

 

pub trait VectorMask: Copy + Clone + PartialEq{
    type Array: Copy;
    
    fn any(self) -> bool;
    fn all(self) -> bool;
    
    fn from_array(arr: Self::Array) -> Self;
    fn to_array(self) -> Self::Array;
}

macro_rules! impl_vector_mask {
    ($mask_type:ty, $dim:expr) => {
        impl VectorMask for $mask_type {
            type Array = [bool; $dim];

            #[inline] 
            fn any(self) -> bool { 
                // Use explicit trait/method qualification via standard inherent bitmask calls
                // or fall back to native dot notation if the type implements it natively
                self.any()
            }

            #[inline] 
            fn all(self) -> bool { 
                self.all()
            }

            #[inline] 
            fn from_array(arr: Self::Array) -> Self { 
                // Use fully qualified path syntax to ensure we don't recurse
                <$mask_type>::from_array(arr) 
            }

            #[inline] 
            fn to_array(self) -> Self::Array { 
                // 🟢 FIX: Manually construct the boolean array to completely 
                // bypass name collision and avoid infinite recursion loops.
                let mut bit_arr = [false; $dim];
                for i in 0..$dim {
                    bit_arr[i] = self.test(i); // Or use self.to_bitmask(), depending on your SIMD API
                }
                bit_arr
            }
        }
    };
}
 
pub trait FloatScalar:
    Copy + PartialOrd + Add<Output = Self> + Sub<Output = Self> 
    + Mul<Output = Self> + Div<Output = Self> + Neg<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
    const INFINITY: Self;      
    const NEG_INFINITY: Self;  

    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn from_f64(v: f64) -> Self;
    fn to_f64(self) -> f64;  
    fn abs(self) -> Self;
}

 
macro_rules! impl_float_scalar {
    ($scalar_type:ty) => {
        impl FloatScalar for $scalar_type {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const INFINITY: Self = <$scalar_type>::INFINITY;          
            const NEG_INFINITY: Self = <$scalar_type>::NEG_INFINITY; 

            #[inline] 
            fn sqrt(self) -> Self { 
                <$scalar_type>::sqrt(self) 
            }

            #[inline] 
            fn exp(self) -> Self { 
                <$scalar_type>::exp(self) 
            }

            #[inline] 
            fn from_f64(v: f64) -> Self { 
                v as $scalar_type 
            }

            #[inline] 
            fn to_f64(self) -> f64 { 
                self as f64 
            }

            #[inline] 
            fn abs(self) -> Self { 
                <$scalar_type>::abs(self) 
            }
        }
    };
}

 
pub trait Vector: 
    Copy 
    + PartialEq  
    + Add<Output = Self> 
    + Sub<Output = Self> 
    + AddAssign 
    + SubAssign 
    + Neg<Output = Self>
    + Mul<Self::Scalar, Output = Self> 
    + Div<Self::Scalar, Output = Self>
    // Enforce that this vector can quantize into its matching integer type
    + QuantizeInto<Self::Quantized, Self::Scalar>
{
    const DIM: usize;
    const ZERO: Self;

    type Scalar: FloatScalar; 
    type Mask: VectorMask; 
    
    // The exact matching integer vector type (e.g., IVec2 for DVec2)
    type Quantized; 

    fn dot(self, other: Self) -> Self::Scalar;
    fn splat(v: Self::Scalar) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn div_elementwise(self, other: Self) -> Self;
    fn mul_elementwise(self, other: Self) -> Self;

    fn length_squared(self) -> Self::Scalar;
    fn length(self) -> Self::Scalar;

    // fn cmpeq(self, other: Self) -> Self::Mask; 
    // fn cmpneq(self, other: Self) -> Self::Mask;
    fn cmplt(self, other: Self) -> Self::Mask;
    fn cmpgt(self, other: Self) -> Self::Mask;

    fn select(mask: Self::Mask, true_val: Self, false_val: Self) -> Self;
    fn mask_and(lhs: Self::Mask, rhs: Self::Mask) -> Self::Mask;
    fn mask_or(lhs: Self::Mask, rhs: Self::Mask) -> Self::Mask;
    

    fn from_slice(slice: &[Self::Scalar]) -> Self;
    fn from_f64_array<const N: usize>(arr: [f64; N]) -> Self ;

    #[inline]
    fn contains_point(self, min_bound: Self, max_bound: Self) -> bool {
        let out_min = self.cmplt(min_bound);
        let out_max = self.cmpgt(max_bound);
        !Self::mask_or(out_min, out_max).any()
    }
 
    
}

// ==========================================
// 4. Automated Macro Engine (Using Your Aliases)
// ==========================================

macro_rules! impl_vector_for_alias {
    ($vector_type:ty, $dim:expr, $scalar:ty, $native_mask:ty, $target_mask:ty, $quantized_type:ty) => {
        impl Vector for $vector_type {
            const DIM: usize = $dim;
            const ZERO: Self = <$vector_type>::ZERO;
            type Scalar = $scalar;
            type Mask = $target_mask;
            type Quantized = $quantized_type;

            #[inline] fn dot(self, other: Self) -> Self::Scalar { <$vector_type>::dot(self, other) }
            #[inline] fn splat(v: Self::Scalar) -> Self { <$vector_type>::splat(v) }
            #[inline] fn min(self, other: Self) -> Self { self.min(other) }
            #[inline] fn max(self, other: Self) -> Self { self.max(other) }
            #[inline] fn div_elementwise(self, other: Self) -> Self { self / other }
            #[inline] fn mul_elementwise(self, other: Self) -> Self { self * other }

             #[inline] 
            fn from_slice(slice: &[Self::Scalar]) -> Self { 
                <$vector_type>::from_slice(slice) 
            }

             #[inline]
            fn from_f64_array<const N: usize>(arr: [f64; N]) -> Self {
                let mut components = [<$scalar as FloatScalar>::ZERO; $dim];
                let limit = if N < $dim { N } else { $dim };
                for i in 0..limit {
                    components[i] = <$scalar as FloatScalar>::from_f64(arr[i]);
                }
                <$vector_type>::from_slice(&components)
            }

             /// Calculates the squared length of the vector. 
            /// Fast because it avoids a square root operation.
            #[inline]
            fn length_squared(self) -> Self::Scalar {
                self.dot(self)
            }

            /// Calculates the actual magnitude of the vector.
            /// Requires an expensive square root operation.
            #[inline]
            fn length(self) -> Self::Scalar {
                self.length_squared().sqrt()
            }
            
            // #[inline] fn cmpeq(self, other: Self) -> Self::Mask { 
            //     let native: $native_mask = <$vector_type>::cmpeq(self, other);
            //     // Fully qualified trait calls fix ambiguous method resolutions
            //     let arr = <$native_mask as VectorMask>::to_array(native);
            //     <Self::Mask as VectorMask>::from_array(arr)
            // }
            
            // #[inline] fn cmpneq(self, other: Self) -> Self::Mask { 
            //     let native: $native_mask = <$vector_type>::cmpneq(self, other);
            //     let arr = <$native_mask as VectorMask>::to_array(native);
            //     <Self::Mask as VectorMask>::from_array(arr)
            // }
            
            #[inline] fn cmplt(self, other: Self) -> Self::Mask { 
                let native: $native_mask = <$vector_type>::cmplt(self, other);
                let arr = <$native_mask as VectorMask>::to_array(native);
                <Self::Mask as VectorMask>::from_array(arr)
            }
            
            #[inline] fn cmpgt(self, other: Self) -> Self::Mask { 
                let native: $native_mask = <$vector_type>::cmpgt(self, other);
                let arr = <$native_mask as VectorMask>::to_array(native);
                <Self::Mask as VectorMask>::from_array(arr)
            }
            
            #[inline] fn select(mask: Self::Mask, true_val: Self, false_val: Self) -> Self { 
                let arr = <Self::Mask as VectorMask>::to_array(mask);
                let native = <$native_mask as VectorMask>::from_array(arr);
                <$vector_type>::select(native, true_val, false_val) 
            }
            
            #[inline] fn mask_and(lhs: Self::Mask, rhs: Self::Mask) -> Self::Mask { 
                let l_arr = <Self::Mask as VectorMask>::to_array(lhs);
                let r_arr = <Self::Mask as VectorMask>::to_array(rhs);
                let native_l = <$native_mask as VectorMask>::from_array(l_arr);
                let native_r = <$native_mask as VectorMask>::from_array(r_arr);
                let combined = native_l & native_r;
                <Self::Mask as VectorMask>::from_array(<$native_mask as VectorMask>::to_array(combined))
            }
            
            #[inline] fn mask_or(lhs: Self::Mask, rhs: Self::Mask) -> Self::Mask { 
                let l_arr = <Self::Mask as VectorMask>::to_array(lhs);
                let r_arr = <Self::Mask as VectorMask>::to_array(rhs);
                let native_l = <$native_mask as VectorMask>::from_array(l_arr);
                let native_r = <$native_mask>::from_array(r_arr); // directly works or qualify
                let combined = native_l | native_r;
                <Self::Mask as VectorMask>::from_array(<$native_mask as VectorMask>::to_array(combined))
            }
        }
    };
}

 

 
impl_float_scalar!(f32);
impl_float_scalar!(f64);

impl_vector_mask!(BVec2, 2);
impl_vector_mask!(BVec3, 3);
impl_vector_mask!(BVec4, 4);

// Base quantization logic implementations
impl_quantize!(Vec2, IVec2, f32, [x, y]);
impl_quantize!(DVec2, IVec2, f64, [x, y]);
impl_quantize!(Vec3, IVec3, f32, [x, y, z]);
impl_quantize!(DVec3, IVec3, f64, [x, y, z]);

// Vector trait bindings mapped to their exact Quantized matches
impl_vector_for_alias!(Vec2,  2, f32, BVec2,       BVec2, IVec2);
impl_vector_for_alias!(DVec2, 2, f64, BVec2,       BVec2, IVec2);
impl_vector_for_alias!(Vec3,  3, f32, BVec3,       BVec3, IVec3);
impl_vector_for_alias!(DVec3, 3, f64, BVec3,       BVec3, IVec3);

 

