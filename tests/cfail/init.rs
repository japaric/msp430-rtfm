#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm;
extern crate msp430g2553;

use rtfm::app;

app! { //~ error mismatched types
    device: msp430g2553,
}

// ERROR `init` must have signature `fn (init::Peripherals)`
fn init() {}

fn idle() -> ! {
    loop {}
}
