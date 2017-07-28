//! An application with one task
//!
//! ```
//! #![deny(unsafe_code)]
//! #![feature(abi_msp430_interrupt)]
//! #![feature(const_fn)]
//! #![feature(proc_macro)]
//! #![no_std]
//! 
//! extern crate msp430_rtfm as rtfm;
//! extern crate msp430g2553;
//! 
//! use rtfm::{app, Threshold};
//! 
//! app! {
//!     device: msp430g2553,
//! 
//!     // Here data resources are declared
//!     //
//!     // Data resources are static variables that are safe to share across tasks
//!     resources: {
//!         // Declaration of resources looks exactly like declaration of static
//!         // variables
//!         static ON: bool = false;
//!     },
//! 
//!     // Here tasks are declared
//!     //
//!     // Each task corresponds to an interrupt. Every time the interrupt becomes
//!     // *pending* the corresponding *task handler* will be executed.
//!     tasks: {
//!         // Here we declare that we'll use the TIMER0_A1 interrupt as a task
//!         TIMER0_A1: {
//!             // Path to the task handler
//!             path: timer0_a1,
//! 
//!             // These are the resources this task has access to
//!             //
//!             // A resource can be a peripheral like `PORT_1_2` or a static
//!             // variable like `ON`
//!             resources: [ON, PORT_1_2],
//!         },
//!     }
//! }
//! 
//! fn init(_p: init::Peripherals, _r: init::Resources) {
//!     // .. configure some pin as output ..
//! 
//!     // .. configure the timer to generate one interrupt every second ..
//! }
//! 
//! fn idle() -> ! {
//!     loop {}
//! }
//! 
//! // This is the task handler of the TIMER0_A1 interrupt
//! //
//! // `_t` is the preemption threshold token. We won't use it in this program.
//! //
//! // `r` is the set of resources this task has access to. `TIMER0_A1::Resources`
//! // has one field per resource declared in `app!`.
//! fn timer0_a1(_t: &mut Threshold, r: TIMER0_A1::Resources) {
//!     // toggle state
//!     **r.ON = !**r.ON;
//! 
//!     if **r.ON {
//!         // set the pin high
//!     } else {
//!         // set the pin low
//!     }
//! }
//! ```
// Auto-generated. Do not modify.
