pub trait Sealed {}

impl<T: Sealed, const N: usize> Sealed for [T; N] {}
impl<T: Sealed> Sealed for [T] {}

impl<T: Sealed + ?Sized> Sealed for &T {}
impl<T: Sealed + ?Sized> Sealed for &mut T {}

macro_rules! seal {
    (@int $($ty:ty),* $(,)?) => {
        $crate::sealed::seal!($($ty),*);

        $(
            impl $crate::sealed::Sealed for ::core::num::NonZero::<$ty> {}
        )*
    };
    ($($ty:ty),* $(,)?) => {
        $(
            impl $crate::sealed::Sealed for $ty {}
        )*
    };
}

pub(crate) use seal;

seal!(@int u8, u16, u32, u64, u128, usize);
seal!(@int i8, i16, i32, i64, i128, isize);
seal!(f32, f64);
seal!(());
seal!(char);
seal!(bool);
seal!(str);
