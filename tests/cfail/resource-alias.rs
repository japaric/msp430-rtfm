#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::app;

app! { //~ error proc macro panicked
    device: msp430g2553,

    resources: {
        // resource `MAX` listed twice
        MAX: u8 = 0;
        MAX: u16 = 0;
    },

    tasks: {
        TIMER0_A0: {
            path: timer0_a0,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}
