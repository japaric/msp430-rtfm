//! A showcase of the `app!` macro syntax
#![deny(unsafe_code)]
#![feature(abi_msp430_interrupt)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::{app, Threshold};

app! {
    device: msp430g2553,

    resources: {
        static CO_OWNED: u32 = 0;
        static ON: bool = false;
        static OWNED: bool = false;
        static SHARED: bool = false;
    },

    init: {
        // This is the path to the `init` function
        //
        // `init` doesn't necessarily has to be in the root of the crate
        path: main::init,
    },

    idle: {
        // This is a path to the `idle` function
        //
        // `idle` doesn't necessarily has to be in the root of the crate
        path: main::idle,
        resources: [OWNED, SHARED],
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            resources: [CO_OWNED, ON, SHARED],
        },

        TIMER0_A1: {
            path: timer0_a1,
            resources: [CO_OWNED],
        },
    },
}

mod main {
    use rtfm::{Resource, Threshold};

    pub fn init(_p: ::init::Peripherals, _r: ::init::Resources) {}

    pub fn idle(t: &mut Threshold, mut r: ::idle::Resources) -> ! {
        loop {
            *r.OWNED != *r.OWNED;

            if *r.OWNED {
                if r.SHARED.claim(t, |shared, _| **shared) {
                    // ..
                }
            } else {
                r.SHARED.claim_mut(t, |shared, _| **shared = !**shared);
            }
        }
    }
}

fn timer0_a0(_t: &mut Threshold, r: TIMER0_A0::Resources) {
    **r.ON = !**r.ON;

    **r.CO_OWNED += 1;
}

fn timer0_a1(_t: &mut Threshold, r: TIMER0_A1::Resources) {
    **r.CO_OWNED += 1;
}
