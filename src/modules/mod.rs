pub mod input_injector;
pub mod register;

pub use input_injector::InputInjectorModule;
pub use register::RegisterResetModule;
use serde::{Deserialize, Serialize};
// use std::cell::UnsafeCell;
// use libafl_qemu::modules::NopAddressFilter;

// static mut NOP_ADDRESS_FILTER: UnsafeCell<NopAddressFilter> = UnsafeCell::new(NopAddressFilter);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExecMeta {
    pub ignore: bool,
}

impl ExecMeta {
    pub fn new() -> Self {
        Self { ignore: false }
    }
}

libafl_bolts::impl_serdeany!(ExecMeta);