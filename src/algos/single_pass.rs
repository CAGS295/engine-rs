use core::ops::{Add, Div, Mul, Sub};

pub fn mean<T>(last: T, new: impl Into<T>, size: u32) -> T
where
  T: Add<Output = T> + Sub<Output = T> + Div<Output = T> + Copy + From<u32>,
{
  last + (new.into() - last) / size.into()
}

pub fn sum<T: Add<T, Output = T>>(values: impl Iterator<Item = T>) -> T
where
  T: Zero,
{
  values.fold(Zero::zero(), |acc, x| acc + x)
}

pub fn mean_centered_sum_squared<T>(
  values: impl Iterator<Item = T>,
  mean: f64,
) -> T
where
  T: Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
  T: Zero,
  T: Copy,
  T: From<f64>,
{
  values.fold(Zero::zero(), |acc, x| {
    let diff = x - T::from(mean);
    acc + diff * diff
  })
}

pub trait Zero {
  fn zero() -> Self;
}

impl Zero for f64 {
  fn zero() -> Self {
    0f64
  }
}
