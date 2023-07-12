//! This crate provides traits for working with finite fields.

// Catch documentation errors caused by code changes.
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(unused_imports)]

pub use ff_uint_derive::*;

#[cfg(feature = "rand_support")]
use rand_core::RngCore;

pub mod traits;

pub use self::arith_impl::*;

pub mod arith_impl {
    #[cfg(target_arch = "wasm32")]
    #[inline(always)]
    pub fn mul_u64(x: u64, y: u64) -> u128 {
        // x = x_1 * 2^32 + x_0
        let x_0 = (x as u32) as u64;
        let x_1 = x >> 32;

        // y = y_1 * 2^32 + y_0
        let y_0 = (y as u32) as u64;
        let y_1 = y >> 32;

        let z_0 = x_0 * y_0;
        let z_2 = x_1 * y_1;

        let z_1 = u128::from(x_0 * y_1) + u128::from(x_1 * y_0);

        (u128::from(z_2) << 64) + (z_1 << 32) + u128::from(z_0)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[inline(always)]
    pub fn mul_u64(x: u64, y: u64) -> u128 {
        (x as u128) * (y as u128)
    }

    /// Calculate a - b - borrow, returning the result and modifying
    /// the borrow value.
    #[inline(always)]
    pub fn sbb(a: u64, b: u64, borrow: &mut u64) -> u64 {
        let tmp = (1u128 << 64) + u128::from(a) - u128::from(b) - u128::from(*borrow);

        *borrow = if tmp >> 64 == 0 { 1 } else { 0 };

        tmp as u64
    }

    /// Calculate a + b + carry, returning the sum and modifying the
    /// carry value.
    #[inline(always)]
    pub fn adc(a: u64, b: u64, carry: &mut u64) -> u64 {
        let tmp = u128::from(a) + u128::from(b) + u128::from(*carry);

        *carry = (tmp >> 64) as u64;

        tmp as u64
    }

    /// Calculate a + (b * c) + carry, returning the least significant digit
    /// and setting carry to the most significant digit.
    #[inline(always)]
    pub fn mac_with_carry(a: u64, b: u64, c: u64, carry: &mut u64) -> u64 {
        let tmp = u128::from(a) + mul_u64(b, c) + u128::from(*carry);

        *carry = (tmp >> 64) as u64;

        tmp as u64
    }
}
