use libafl::inputs::UsesInput;
use libafl_qemu::{
    modules::{EmulatorModule, EmulatorModuleTuple, NopAddressFilter}, EmulatorModules, Qemu
};

#[derive(Default, Debug)]
pub struct RegisterResetModule {
    regs: Vec<u64>,
}

impl RegisterResetModule {
    pub fn new(qemu: &Qemu) -> Self {
        log::info!("Saving register state at start point ...");

        // BUG: qemu.num_regs()
        let regs = (0..qemu.num_regs())
            .map(|i| qemu.read_reg(i).unwrap_or(0))
            .collect::<Vec<u64>>();

        // print x22 reg in regs
        log::info!("Saving x22: {:#x}", qemu.read_reg(22).unwrap_or(0));
        assert_eq!(qemu.read_reg(22).unwrap_or(0), regs[22]);

        Self { regs }
    }

    fn restore(&self, qemu: &Qemu) {
        self.regs.iter()
            .enumerate()
            .for_each(|(reg_idx, reg_val)| {
                if let Err(_) = qemu.write_reg(reg_idx as i32, *reg_val) {
                    log::error!("Failed to restore register {}, skipping ...", reg_idx);
                }
            });

        log::info!("Restoring x22: {:#x}", qemu.read_reg(22).unwrap_or(0));
    }
}

impl<S> EmulatorModule<S> for RegisterResetModule
where
    S: UsesInput,
{   
    type ModuleAddressFilter = NopAddressFilter;

    fn pre_exec<ET>(
        &mut self,
        _emulator_modules: &mut EmulatorModules<ET, S>,
        _state: &mut S,
        _input: &S::Input,
    ) where
        ET: EmulatorModuleTuple<S>,
    {
        log::info!("Running register reset module pre-exec");
        self.restore(&_emulator_modules.qemu());
    }

    fn address_filter(&self) -> &Self::ModuleAddressFilter {
        &NopAddressFilter
    }

    fn address_filter_mut(&mut self) -> &mut Self::ModuleAddressFilter {
        // unsafe { (&raw mut NOP_ADDRESS_FILTER).as_mut().unwrap().get_mut() }
        unimplemented!("This should never be called")
    }
}