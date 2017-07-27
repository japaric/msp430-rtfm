#![deny(warnings)]
#![feature(abi_msp430_interrupt)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::{app, Resource, Threshold};

app! {
    device: msp430g2553,

    resources: {
        static STATE: bool = false;
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            resources: [STATE],
        },

        TIMER0_A1: {
            path: timer0_a1,
            resources: [STATE],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn timer0_a0(mut t: &mut Threshold, r: TIMER0_A0::Resources) {
    // ERROR token should not outlive the critical section
    let t = r.STATE.claim(&mut t, |_state, t| t);
    //~^ error cannot infer an appropriate lifetime
}

fn timer0_a1(_t: &mut Threshold, _r: TIMER0_A1::Resources) {}
