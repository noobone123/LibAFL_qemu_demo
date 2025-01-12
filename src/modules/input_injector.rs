use std::process::abort;

use libafl::inputs::{BytesInput, HasTargetBytes, UsesInput};
use libafl_qemu::{
    modules::{utils::filters::NopAddressFilter, EmulatorModule, EmulatorModuleTuple}, EmulatorModules, GuestAddr, Hook, Qemu, SYS_exit, SYS_exit_group, SYS_mmap, SYS_munmap, SYS_read, SyscallHookResult
};

#[derive(Default, Debug)]
pub struct InputInjectorModule {
    // Save the Mutator's BytesInput
    input: Vec<u8>,
    input_addr: GuestAddr,
    max_size: usize,
}

impl InputInjectorModule {
    pub fn new() -> Self {
        Self {
            max_size: 1048576,
            ..Default::default()
        }
    }

    pub fn set_input_addr(&mut self, addr: GuestAddr) {
        self.input_addr = addr;
    }
}

impl<S> EmulatorModule<S> for InputInjectorModule
where
    S: Unpin + UsesInput<Input = BytesInput>,
{
    type ModuleAddressFilter = NopAddressFilter;

    fn first_exec<ET>(
        &mut self,
        _qemu: Qemu,
        _emulator_modules: &mut EmulatorModules<ET, S>,
        _state: &mut S,
    ) where
        ET: EmulatorModuleTuple<S>,
    {
        log::info!("InputInjectorModule::first_exec running ...");

        if let Some(hook_id) =
            _emulator_modules.pre_syscalls(Hook::Function(syscall_hooks::<ET, S>))
        {
            log::info!("Hook {:?} installed", hook_id);
        } else {
            log::error!("Failed to install hook");
        }
    }

    fn pre_exec<ET>(
        &mut self,
        _qemu: Qemu,
        _emulator_modules: &mut EmulatorModules<ET, S>,
        _state: &mut S,
        _input: &S::Input,
    ) where
        ET: EmulatorModuleTuple<S>,
    {
        let mut tb = _input.target_bytes();
        if tb.len() > self.max_size {
            if let None = tb.truncate(self.max_size) {
                log::error!("Failed to truncate input");
                return;
            }
        }

        log::info!("Injecting input of size {} at address {:#x}", tb.len(), self.input_addr);
        self.input.clear();
        self.input.extend_from_slice(&tb);

        // clean and fill the input_addr for further mmap usage
        let written_buf = if self.input.len() > self.max_size {
            &self.input[..self.max_size]
        } else {
            &self.input
        };
        _qemu.write_mem(self.input_addr, written_buf).unwrap();
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
    S: Unpin + UsesInput<Input = BytesInput>,
    ET: EmulatorModuleTuple<S>,
{
    let sys_num = sys_num as i64;
    // Hook syscall read
    if sys_num == SYS_read {
        log::info!("Read syscall intercepted ...");
        let input_injector_module = emulator_modules
            .get_mut::<InputInjectorModule>()
            .expect("Failed to get InputInjectorModule");
        
        let input_len = input_injector_module.input.len();
        let offset: usize = if _a2 == 0 {
            0
        } else if _a2 as usize <= input_len {
            _a2.try_into().unwrap()
        } else {
            input_len
        };

        let drained = input_injector_module
            .input
            .drain(..offset)
            .as_slice().to_owned();

        _qemu.write_mem(a1, drained.as_slice()).unwrap();

        // Return the number of bytes read
        SyscallHookResult::new(Some(drained.len() as u64))
    }
    else if sys_num == SYS_mmap {
        if _a2 == 1 && _a3 == 1 {
            log::info!("Mmap syscall intercepted ...");
            let input_injector_module = emulator_modules
                .get_mut::<InputInjectorModule>()
                .expect("Failed to get InputInjectorModule");
            log::info!("Mmap return address: {:#x}", input_injector_module.input_addr);
            SyscallHookResult::new(Some(input_injector_module.input_addr))
        } else {
            SyscallHookResult::new(None)
        }
    }
    else if sys_num == SYS_munmap {
        let input_injector_module = emulator_modules
                .get_mut::<InputInjectorModule>()
                .expect("Failed to get InputInjectorModule");
        let addr = input_injector_module.input_addr;
        log::info!("Munmap args: {:#x}, {:#x}", a0, a1);
        if a0 == addr {
            log::info!("Munmap syscall intercepted ...");
            SyscallHookResult::new(Some(0))
        } else {
            SyscallHookResult::new(None)
        }
    }
    else if sys_num == SYS_exit || sys_num == SYS_exit_group {
        log::info!("Exit / Exit group syscall intercepted ...");
        // I tried to abort the process here, however, unlike what I expected, the fuzzer stopped.
        abort();
        // Return SyscallHookResult with None to let the syscall execute normally and program and fuzzer will return
        // SyscallHookResult::new(None)
    }
    else {
        SyscallHookResult::new(None)
    }
}
