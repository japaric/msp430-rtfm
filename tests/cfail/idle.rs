#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::app;

app! { //~ error mismatched types
    device: msp430g2553,
}

fn init(_p: init::Peripherals) {}

// ERROR `idle` must be a diverging function
fn idle() {}
