use noisy_float::types::R32;

/// A custom `f32` alternative.
///
/// This type has a couple of properties that we want to use.
/// 1. It wraps `noisy_float::types::R32`, ensuring that all values are finite
///    and actual numbers in debug mode.
/// 2. It implements the functions from `libm`, which should be cross-platform
///    deterministic.
#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::Neg,
    derive_more::Add,
    derive_more::Sub,
    derive_more::Mul,
    derive_more::Div,
    derive_more::Rem,
    derive_more::Sum,
    derive_more::AddAssign,
    derive_more::SubAssign,
    derive_more::MulAssign,
    derive_more::DivAssign,
    derive_more::RemAssign,
)]
pub struct F32(R32);

pub const PI: F32 = F32::unchecked_new(std::f32::consts::PI);

impl From<f32> for F32 {
    #[inline]
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<R32> for F32 {
    #[inline]
    fn from(value: R32) -> Self {
        Self(value)
    }
}

type Libm = libm::Libm<f32>;

macro_rules! impl_libm_unary {
    () => {};
    ($fun:ident $($tail:tt)*) => {
        #[inline]
        pub fn $fun(self) -> Self {
            Self::new(Libm::$fun(self.0.raw()))
        }
    };
}

impl F32 {
    #[inline]
    pub const fn unchecked_new(float: f32) -> Self {
        Self(R32::unchecked_new(float))
    }

    #[inline]
    pub fn new(float: f32) -> Self {
        Self(R32::new(float))
    }

    impl_libm_unary! {
        acos acosh asin asinh atan atanh cbrt ceil cos cosh erf erfc exp exp2
        exp10 expm1 fabs floor j0 j1 lgamma log log1p log2 log10 rint round sin
        sinh sqrt tan tanh tgamma trunc y0 y1
    }

    #[inline]
    pub fn atan2(y: Self, x: Self) -> Self {
        Self::new(Libm::atan2(y.0.raw(), x.0.raw()))
    }
    #[inline]
    pub fn copysign(x: Self, y: Self) -> Self {
        Self::new(Libm::copysign(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn fdim(x: Self, y: Self) -> Self {
        Self::new(Libm::fdim(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn fma(x: Self, y: Self, z: Self) -> Self {
        Self::new(Libm::fma(x.0.raw(), y.0.raw(), z.0.raw()))
    }
    #[inline]
    pub fn fmax(x: Self, y: Self) -> Self {
        Self::new(Libm::fmax(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn fmin(x: Self, y: Self) -> Self {
        Self::new(Libm::fmin(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn fmod(x: Self, y: Self) -> Self {
        Self::new(Libm::fmod(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn frexp(self) -> (Self, i32) {
        let (f, i) = Libm::frexp(self.0.raw());
        (Self::new(f), i)
    }
    #[inline]
    pub fn hypot(x: Self, y: Self) -> Self {
        Self::new(Libm::hypot(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn ilogb(self) -> i32 {
        Libm::ilogb(self.0.raw())
    }
    #[inline]
    pub fn jn(n: i32, x: Self) -> Self {
        Self::new(Libm::jn(n, x.0.raw()))
    }
    #[inline]
    pub fn ldexp(x: Self, n: i32) -> Self {
        Self::new(Libm::ldexp(x.0.raw(), n))
    }
    #[inline]
    pub fn lgamma_r(self) -> (Self, i32) {
        let (f, i) = Libm::lgamma_r(self.0.raw());
        (Self::new(f), i)
    }
    #[inline]
    pub fn modf(self) -> (Self, Self) {
        let (a, b) = Libm::modf(self.0.raw());
        (Self::new(a), Self::new(b))
    }
    #[inline]
    pub fn nextafter(x: Self, y: Self) -> Self {
        Self::new(Libm::nextafter(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn pow(x: Self, y: Self) -> Self {
        Self::new(Libm::pow(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn remainder(x: Self, y: Self) -> Self {
        Self::new(Libm::remainder(x.0.raw(), y.0.raw()))
    }
    #[inline]
    pub fn remquo(x: Self, y: Self) -> (Self, i32) {
        let (f, i) = Libm::remquo(x.0.raw(), y.0.raw());
        (Self::new(f), i)
    }
    #[inline]
    pub fn scalbn(x: Self, n: i32) -> Self {
        Self::new(Libm::scalbn(x.0.raw(), n))
    }
    #[inline]
    pub fn sincos(self) -> (Self, Self) {
        let (sin, cos) = Libm::sincos(self.0.raw());

        (Self::new(sin), Self::new(cos))
    }
    #[inline]
    pub fn yn(n: i32, x: Self) -> Self {
        Self::new(Libm::yn(n, x.0.raw()))
    }
}
