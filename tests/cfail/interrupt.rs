#![deny(warnings)]
#![feature(abi_msp430_interrupt)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::app;

app! { //~ error no associated item named `TIMER3_A3` found for type
    device: msp430g2553,

    tasks: {
        // ERROR this interrupt doesn't exist
        TIMER3_A3: {
            path: timer3_a3,
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {}
}

fn timer3_a3() {}
