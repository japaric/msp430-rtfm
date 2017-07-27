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
        static MAX: u8 = 0;
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            resources: [MAX, ON],
        },

        TIMER0_A1: {
            path: timer0_a1,
            resources: [ON],
        },

        TIMER1_A0: {
            path: timer1_a0,
            resources: [MAX],
        },
    },
}

fn init(_p: init::Peripherals, _r: init::Resources) {}

fn idle() -> ! {
    loop {}
}

fn timer0_a0(mut t: &mut Threshold, mut r: TIMER0_A0::Resources) {
    // OK need to lock to access the resource
    if r.ON.claim(&mut t, |on, _| **on) {}

    // OK can claim a resource with maximum ceiling
    r.MAX.claim_mut(&mut t, |max, _| **max += 1);
}

fn timer0_a1(mut t: &mut Threshold, r: TIMER0_A1::Resources) {
    // ERROR no need to lock. Has direct access because priority == ceiling
    if (**r.ON).claim(&mut t, |on, _| **on) {
        //~^ error no method named `claim` found for type
    }

    if **r.ON {
        // OK
    }
}

fn timer1_a0(_t: &mut Threshold, _r: TIMER1_A0::Resources) {}
