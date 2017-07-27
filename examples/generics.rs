//! Working with resources in a generic fashion
#![deny(unsafe_code)]
#![feature(abi_msp430_interrupt)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::{app, Resource, Threshold};
use msp430g2553::{PORT_1_2, TIMER0_A3};

app! {
    device: msp430g2553,

    idle: {
        resources: [PORT_1_2, TIMER0_A3],
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            resources: [PORT_1_2, TIMER0_A3],
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    work(t, &r.PORT_1_2, &r.TIMER0_A3);

    loop {}
}

// a generic function to use resources in any task (regardless of its priority)
fn work<P, T>(t: &mut Threshold, port1: &P, timer0: &T)
where
    P: Resource<Data = PORT_1_2>,
    T: Resource<Data = TIMER0_A3>,
{
    port1.claim(t, |_port1, _| {
        // ..
    });

    timer0.claim(t, |_timer0, t| {
        // ..

        port1.claim(t, |_port1, _| {
            // ..
        });

        // ..
    });
}

// this task needs critical sections to access the resources
fn timer0_a0(t: &mut Threshold, r: TIMER0_A0::Resources) {
    work(t, r.PORT_1_2, r.TIMER0_A3);
}
