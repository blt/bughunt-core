// For reasons that are beyond me, if you remove this no_std there's
// going to be a segfault in a fmt somewhere. I have not tracked this
// down. Surprising!
// #![no_std]

//! Core library for bughunt

#![cfg_attr(feature = "cargo-clippy", allow(clippy::cargo))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::complexity))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::correctness))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::perf))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[deny(bad_style)]
#[deny(future_incompatible)]
#[deny(missing_docs)]
#[deny(nonstandard_style)]
#[deny(rust_2018_compatibility)]
#[deny(rust_2018_idioms)]
#[deny(unused)]
#[deny(warnings)]
#[no_mangle]

static mut COUNT_TABLE: &'static mut [u8] = &mut [0; 16_384];

/// TODO(blt) -- not sure what 'callee' should be, do not deference
#[no_mangle]
pub extern "C" fn __sanitizer_cov_trace_pc_indir(_callee: i64) -> () {}

/// For every guard encountered, increment a counter in COUNT_TABLE. This table
/// is never returned anywhere and is private to this library but you can
/// confirm that it's incremented in lldb.
#[no_mangle]
pub extern "C" fn __sanitizer_cov_trace_pc_guard(guard_id: *const u32) -> () {
    unsafe {
        if *guard_id == 0 {
            return;
        }
        COUNT_TABLE[(*guard_id as usize) - 1] += 1;
    }
    //
}

/// TODO(blt)
#[no_mangle]
pub extern "C" fn __sanitizer_cov_trace_pc_guard_init(start: *mut u32, stop: *mut u32) -> () {
    // This function sets all pointers between start and stop to 1, 2, 3,
    // 4... The idea is that we'll have unique identifiers for every guard in
    // the executable. It's possible that this function will be called repeately
    // for the same values of start/stop so we bail out early in that case.
    unsafe {
        // bail out if this function has been called before
        if *start != 0 {
            return;
        }
        // loop and set the guard IDs, conveniently 1 indexed, er, indexes
        let mut guard_id: u32 = 1;

        let mut x = start;
        while x <= stop {
            *x = guard_id;
            guard_id += 1;
            x = x.offset(1);
        }
    }
}

pub fn fuzz<F>(closure: F)
where
    F: FnOnce(&[u8]),
{
    let buf: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];
    closure(buf)
}

#[macro_export]
macro_rules! test {
    (|$bytes:ident| $body:block) => {
        bughunt_core::fuzz(|$bytes| $body)
    };
    (|$bytes:ident: &[u8]| $body:block) => {
        bughunt_core::fuzz(|$bytes| $body)
    };
}
