#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::app;

app! { //~ error proc macro panicked
    device: msp430g2553,

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
            // ERROR peripheral appears twice in this list
            resources: [GPIOA, GPIOA],
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}

fn timer0_a0() {}
