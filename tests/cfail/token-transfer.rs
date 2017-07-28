#![deny(warnings)]
#![feature(abi_msp430_interrupt)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::{app, Threshold};

app! { //~ error bound `rtfm::Threshold: core::marker::Send` is not satisfied
    device: msp430g2553,

    resources: {
        static TOKEN: Option<Threshold> = None;
    },

    idle: {
        resources: [TOKEN],
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            resources: [TOKEN],
        },
    }
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn timer0_a0(_t: &mut Threshold, _r: TIMER0_A0::Resources) {}
