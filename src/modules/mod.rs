pub mod input_injector;
pub mod register;

pub use input_injector::InputInjectorModule;
pub use register::RegisterResetModule;
// use std::cell::UnsafeCell;
// use libafl_qemu::modules::NopAddressFilter;

// static mut NOP_ADDRESS_FILTER: UnsafeCell<NopAddressFilter> = UnsafeCell::new(NopAddressFilter);
