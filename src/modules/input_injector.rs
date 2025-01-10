use libafl::inputs::UsesInput;
use libafl_qemu::{
    modules::{utils::filters::NopAddressFilter, EmulatorModule, EmulatorModuleTuple},
    EmulatorModules, GuestAddr, Hook, Qemu, SYS_read, SyscallHookResult,
};

#[derive(Default, Debug)]
pub struct InputInjectorModule {
    // Save the Mutator's BytesInput
    input: Vec<u8>,
    input_addr: GuestAddr,
}

impl InputInjectorModule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_input_addr(&mut self, addr: GuestAddr) {
        self.input_addr = addr;
    }
}

impl<S> EmulatorModule<S> for InputInjectorModule
where
    S: Unpin + UsesInput,
{
    type ModuleAddressFilter = NopAddressFilter;

    fn first_exec<ET>(
            &mut self,
            _qemu: Qemu,
            _emulator_modules: &mut EmulatorModules<ET, S>,
            _state: &mut S,
        ) where
            ET: EmulatorModuleTuple<S>, {

        log::info!("InputInjectorModule::first_exec running ...");

        if let Some(hook_id) =
            _emulator_modules.pre_syscalls(Hook::Function(syscall_hooks::<ET, S>))
        {
            log::info!("Hook {:?} installed", hook_id);
        } else {
            log::error!("Failed to install hook");
        }
    }

    fn address_filter(&self) -> &Self::ModuleAddressFilter {
        &NopAddressFilter
    }

    fn address_filter_mut(&mut self) -> &mut Self::ModuleAddressFilter {
        // unsafe { (&raw mut NOP_ADDRESS_FILTER).as_mut().unwrap().get_mut() }
        unimplemented!("This should never be called")
    }
}

/// This is user-defined syscall hook.
/// If create `SyscallHookResult` with `None`, the syscall will execute normally
/// If create `SyscallHookResult` with `Some(retval)`, the syscall will directly return the retval and not execute
fn syscall_hooks<ET, S>(
    _qemu: Qemu,
    emulator_modules: &mut EmulatorModules<ET, S>,
    _state: Option<&mut S>,
    sys_num: i32,
    a0: GuestAddr,
    a1: GuestAddr,
    _a2: GuestAddr,
    _a3: GuestAddr,
    _a4: GuestAddr,
    _a5: GuestAddr,
    _a6: GuestAddr,
    _a7: GuestAddr,
) -> SyscallHookResult
where
    S: Unpin + UsesInput,
    ET: EmulatorModuleTuple<S>,
{
    let sys_num = sys_num as i64;
    if sys_num == SYS_read {
        log::info!("Hooked syscall read");
        log::info!("fd = {:#x}, buf = {:#x}, count = {:#x}", a0, a1, _a2);
        SyscallHookResult::new(None)
    } else {
        SyscallHookResult::new(None)
    }
}
