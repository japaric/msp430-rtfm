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
        static ON: bool = false;
    },

    idle: {
        resources: [ON],
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            resources: [ON],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    let state = rtfm::atomic(t, |t| {
        // ERROR borrow can't escape this *global* critical section
        r.ON.borrow(t) //~ error cannot infer an appropriate lifetime
    });

    let state = r.ON.claim(t, |state, _t| {
        // ERROR borrow can't escape this critical section
        state //~ error cannot infer an appropriate lifetime
    });

    loop {}
}

fn timer0_a0(_t: &mut Threshold, _r: TIMER0_A0::Resources) {}
