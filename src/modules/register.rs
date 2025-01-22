use libafl_qemu::{
    modules::{utils::filters::NopAddressFilter, EmulatorModule, EmulatorModuleTuple},
    EmulatorModules, Qemu, QemuParams, Regs,
};

#[derive(Default, Debug)]
pub struct RegisterResetModule {
    reg_num: usize,
    regs: Vec<u64>,
}

impl RegisterResetModule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(&mut self, qemu: Qemu) {
        log::debug!("Saving register state at start point ...");

        self.reg_num = qemu.num_regs() as usize;
        let regs = (0..self.reg_num)
            .map(|i| qemu.read_reg(i as i32).unwrap_or(0))
            .collect::<Vec<u64>>();
        self.regs = regs;
    }

    fn restore(&self, qemu: Qemu) {
        self.regs.iter().enumerate().for_each(|(reg_idx, reg_val)| {
            if let Err(_) = qemu.write_reg(reg_idx as i32, *reg_val) {
                log::error!("Failed to restore register {}, skipping ...", reg_idx);
            }
        });
    }
}

impl<I, S> EmulatorModule<I, S> for RegisterResetModule
where
    S: Unpin,
    I: Unpin,
{
    type ModuleAddressFilter = NopAddressFilter;

    fn pre_qemu_init<ET>(
        &mut self,
        _emulator_modules: &mut EmulatorModules<ET, I, S>,
        _qemu_params: &mut QemuParams,
    ) where
        ET: EmulatorModuleTuple<I, S>,
    {
        log::debug!("RegisterResetModule::pre_qemu_init running ...");
    }

    fn post_qemu_init<ET>(&mut self, _qemu: Qemu, _emulator_modules: &mut EmulatorModules<ET, I, S>)
    where
        ET: EmulatorModuleTuple<I, S>,
    {
        log::debug!("RegisterResetModule::post_qemu_init running ...");
    }

    fn pre_exec<ET>(
        &mut self,
        _qemu: Qemu,
        _emulator_modules: &mut EmulatorModules<ET, I, S>,
        _state: &mut S,
        _input: &I,
    ) where
        ET: EmulatorModuleTuple<I, S>,
    {
        log::debug!("RegisterResetModule::pre_exec running ...");
        self.restore(_qemu);
    }

    fn address_filter(&self) -> &Self::ModuleAddressFilter {
        &NopAddressFilter
    }

    fn address_filter_mut(&mut self) -> &mut Self::ModuleAddressFilter {
        // unsafe { (&raw mut NOP_ADDRESS_FILTER).as_mut().unwrap().get_mut() }
        unimplemented!("This should never be called")
    }
}
