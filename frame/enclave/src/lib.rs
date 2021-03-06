#![no_std]
#[macro_use]
extern crate sgx_tstd as std;

pub mod engine;
mod register;
mod error;
pub mod ocalls;
// mod state_machine;

pub use crate::engine::*;
pub use crate::error::FrameEnclaveError as Error;
