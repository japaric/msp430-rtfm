//! Two tasks running at the same priority with access to the same resource
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

    // Resources that are plain data, not peripherals
    resources: {
        // Declaration of resources looks like the declaration of `static`
        // variables
        static COUNTER: u64 = 0;
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            // Both this task and TIM2 have access to the `COUNTER` resource
            resources: [COUNTER],
        },

        // An interrupt as a task
        TIMER0_A1: {
            // For interrupts the `enabled` field must be specified. It
            // indicates if the interrupt will be enabled or disabled once
            // `idle` starts
            path: timer0_a1,
            resources: [COUNTER],
        },
    },
}

// when data resources are declared in the top `resources` field, `init` will
// have full access to them
fn init(_p: init::Peripherals, _r: init::Resources) {
    // ..
}

fn idle() -> ! {
    loop {}
}

// As both tasks are running at the same priority one can't preempt the other.
// Thus both tasks have direct access to the resource
fn timer0_a0(_t: &mut Threshold, r: TIMER0_A0::Resources) {
    // ..

    **r.COUNTER += 1;

    // ..
}

fn timer0_a1(_t: &mut Threshold, r: TIMER0_A1::Resources) {
    // ..

    **r.COUNTER += 1;

    // ..
}
