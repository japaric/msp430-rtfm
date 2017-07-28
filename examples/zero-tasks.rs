//! Minimal example with zero tasks
#![deny(unsafe_code)]
// IMPORTANT always include this feature gate
#![feature(proc_macro)]
#![no_std]

extern crate msp430_rtfm as rtfm; // IMPORTANT always do this rename
extern crate msp430g2553; // the device crate

// import the procedural macro
use rtfm::app;

// This macro call indicates that this is a RTFM application
//
// This macro will expand to a `main` function so you don't need to supply
// `main` yourself.
app! {
    // this is the path to the device crate
    device: msp430g2553,
}

// The initialization phase.
//
// This runs first and within a *global* critical section. Nothing can preempt
// this function.
fn init(p: init::Peripherals) {
    // This function has access to all the peripherals of the device
    p.PORT_1_2;
    p.SYSTEM_CLOCK;
    // ..
}

// The idle loop.
//
// This runs after `init` and has a priority of 0. All tasks can preempt this
// function. This function can never return so it must contain some sort of
// endless loop.
fn idle() -> ! {
    // Busy wait until there's a task to service
    loop {}
}
