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
        static A: u8 = 0;
        static B: u8 = 0;
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            resources: [A, B],
        },

        TIMER0_A1: {
            path: timer0_a1,
            resources: [A, B],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn timer0_a0(mut ot: &mut Threshold, r: TIMER0_A0::Resources) {
    r.A.claim(&mut ot, |_a, mut _it| {
        //~^ error cannot borrow `ot` as mutable more than once at a time
        //~| error cannot borrow `ot` as mutable more than once at a time
        // ERROR must use inner token `it` instead of the outer one (`ot`)
        r.B.claim(&mut ot, |_b, _| {})
    });
}

fn timer0_a1(_t: &mut Threshold, r: TIMER0_A1::Resources) {}
