// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic support for building blocking abstractions.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::marker::{Sync, Send};
use std::mem;
use std::clone::Clone;
use parking_lot::{park, unpark_one};

struct Inner {
    woken: AtomicBool,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

#[derive(Clone)]
pub struct SignalToken {
    inner: Arc<Inner>,
}

pub struct WaitToken {
    inner: Arc<Inner>,
}

impl !Send for WaitToken {}

impl !Sync for WaitToken {}

pub fn tokens() -> (WaitToken, SignalToken) {
    let inner = Arc::new(Inner {
        woken: AtomicBool::new(false),
    });
    let wait_token = WaitToken {
        inner: inner.clone(),
    };
    let signal_token = SignalToken {
        inner: inner
    };
    (wait_token, signal_token)
}

impl SignalToken {
    pub fn signal(&self) -> bool {
        let wake = !self.inner.woken.compare_and_swap(false, true, Ordering::SeqCst);
        if wake {
            let callback = &mut |_| {
            };
            unsafe { unpark_one(mem::transmute_copy(self), callback); }
        }
        wake
    }

    /// Convert to an unsafe usize value. Useful for storing in a pipe's state
    /// flag.
    #[inline]
    pub unsafe fn cast_to_usize(self) -> usize {
        mem::transmute(self.inner)
    }

    /// Convert from an unsafe usize value. Useful for retrieving a pipe's state
    /// flag.
    #[inline]
    pub unsafe fn cast_from_usize(signal_ptr: usize) -> SignalToken {
        SignalToken { inner: mem::transmute(signal_ptr) }
    }

}

impl WaitToken {
    pub fn wait(self) {
        if !self.inner.woken.load(Ordering::SeqCst) {
            let validate = &mut || {
                !self.inner.woken.load(Ordering::SeqCst)
            };
            let before_sleep = &mut || {
            };
            let timed_out = &mut |_, _| {
            };
            unsafe { park(mem::transmute_copy(&self), validate, before_sleep, timed_out, None); }
        }
    }
}
