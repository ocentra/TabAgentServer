//! Algorithm utilities and numeric traits for graph algorithms.
//!
//! This module provides common utilities needed by graph algorithms,
//! particularly for handling edge weights and costs.

use std::ops::Add;

/// Trait for types that can be used as edge weights in algorithms.
///
/// This trait defines the requirements for types that represent measurements
/// or costs in graph algorithms like Dijkstra's algorithm.
pub trait Measure: Add<Output = Self> + PartialOrd + Clone + Default {
    /// Returns the zero/identity value for this measure.
    fn zero() -> Self {
        Self::default()
    }
}

// Implement Measure for common numeric types
impl Measure for u8 {}
impl Measure for u16 {}
impl Measure for u32 {}
impl Measure for u64 {}
impl Measure for u128 {}
impl Measure for usize {}
impl Measure for i8 {}
impl Measure for i16 {}
impl Measure for i32 {}
impl Measure for i64 {}
impl Measure for i128 {}
impl Measure for isize {}
impl Measure for f32 {}
impl Measure for f64 {}

/// A type representing infinity or unbounded distance.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Infinity;

/// Error type for negative cycles in shortest path algorithms.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NegativeCycle;

impl std::fmt::Display for NegativeCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "graph contains a negative cycle")
    }
}

impl std::error::Error for NegativeCycle {}

pub trait BoundedMeasure: Measure + core::ops::Sub<Self, Output = Self> {
    fn min() -> Self;
    fn max() -> Self;
    fn overflowing_add(self, rhs: Self) -> (Self, bool);
    fn from_f32(val: f32) -> Self;
    fn from_f64(val: f64) -> Self;
}

macro_rules! impl_bounded_measure_integer {
    ( $( $t:ident ),* ) => {
        $(
            impl BoundedMeasure for $t {
                fn min() -> Self { $t::MIN }
                fn max() -> Self { $t::MAX }
                fn overflowing_add(self, rhs: Self) -> (Self, bool) { self.overflowing_add(rhs) }
                fn from_f32(val: f32) -> Self { val as $t }
                fn from_f64(val: f64) -> Self { val as $t }
            }
        )*
    };
}

impl_bounded_measure_integer!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! impl_bounded_measure_float {
    ( $( $t:ident ),* ) => {
        $(
            impl BoundedMeasure for $t {
                fn min() -> Self { $t::MIN }
                fn max() -> Self { $t::MAX }
                fn overflowing_add(self, rhs: Self) -> (Self, bool) { (self + rhs, false) }
                fn from_f32(val: f32) -> Self { val as $t }
                fn from_f64(val: f64) -> Self { val as $t }
            }
        )*
    };
}

impl_bounded_measure_float!(f32, f64);

pub trait PositiveMeasure: Measure + Copy {
    fn zero() -> Self;
    fn max() -> Self;
}

macro_rules! impl_positive_measure {
    ( $( $t:ident ),* ) => {
        $(
            impl PositiveMeasure for $t {
                fn zero() -> Self { 0 as $t }
                fn max() -> Self { $t::MAX }
            }
        )*
    };
}

impl_positive_measure!(u8, u16, u32, u64, u128, usize, f32, f64);

pub trait UnitMeasure {}
impl UnitMeasure for f32 {}
impl UnitMeasure for f64 {}

pub trait FloatMeasure: Measure + Copy + std::ops::Div<Output = Self> + std::ops::Sub<Output = Self> + std::ops::Mul<Output = Self> + std::iter::Sum + PartialEq {
    fn one() -> Self;
    fn infinite() -> Self;
    fn from_f32(val: f32) -> Self;
    fn from_f64(val: f64) -> Self;
    fn from_usize(val: usize) -> Self;
    fn default_tol() -> Self;
}

impl FloatMeasure for f32 {
    fn one() -> Self { 1. }
    fn infinite() -> Self { 1. / 0. }
    fn from_f32(val: f32) -> Self { val }
    fn from_f64(val: f64) -> Self { val as f32 }
    fn from_usize(val: usize) -> Self { val as f32 }
    fn default_tol() -> Self { 1e-6 }
}

impl FloatMeasure for f64 {
    fn one() -> Self { 1. }
    fn infinite() -> Self { 1. / 0. }
    fn from_f32(val: f32) -> Self { val as f64 }
    fn from_f64(val: f64) -> Self { val }
    fn from_usize(val: usize) -> Self { val as f64 }
    fn default_tol() -> Self { 1e-10 }
}

