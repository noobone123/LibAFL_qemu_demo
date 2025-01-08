use libafl::{
    executors::ExitKind,
    inputs::{BytesInput, HasTargetBytes},
    Error,
};
use libafl_bolts::AsSlice;
use libafl_qemu::{
    elf::EasyElf, ArchExtras, CallingConvention, GuestAddr, GuestReg, MmapPerms, Qemu, Regs, QemuExitReason
};

pub struct Harness {
    qemu: Qemu,
    input_addr: GuestAddr,
    pc: GuestAddr,
    stack_ptr: GuestAddr,
    ret_addr: GuestAddr,
}

pub const MAX_INPUT_SIZE: usize = 1_048_576; // 1MB

impl Harness {
    /// Change environment
    #[inline]
    #[expect(clippy::ptr_arg)]
    pub fn edit_env(_env: &mut Vec<(String, String)>) {}

    /// Change arguments
    #[inline]
    #[expect(clippy::ptr_arg)]
    pub fn edit_args(_args: &mut Vec<String>) {}

    pub fn read_mem_8(&self, addr: GuestAddr, buf: &mut [u8]) -> Result<(), Error> {
        self.qemu
            .read_mem(addr, buf)
            .map_err(|e| Error::unknown(format!("Failed to read memory@{addr:#x}: {e:?}")))
    }

    /// Initialize the emulator, run to the entrypoint (or jump there) and return the [`Harness`] struct
    pub fn init(qemu: Qemu) -> Result<Harness, Error> {
        let mut elf_buffer = Vec::new();
        let elf = EasyElf::from_file(qemu.binary_path(), &mut elf_buffer)?;
        
        let load_addr = qemu.load_addr();
        log::info!("load_addr = {load_addr:#x}");

        let main_addr = elf
            .resolve_symbol("main", qemu.load_addr())
            .ok_or_else(|| Error::empty_optional("Symbol main not found"))?;

        let start_pc = main_addr + 0x178;
        let end_pc = main_addr + 0x144;

        log::info!("start_pc @ {start_pc:#x}");
        log::info!("end_pc @ {end_pc:#x}");

        // qemu.entry_break(start_pc);
        qemu.set_breakpoint(start_pc);
        unsafe {
            match qemu.run() {
                // It seems that the control will back after the inst at breakpoint addr is executed
                Ok(QemuExitReason::Breakpoint(_)) => {
                    log::info!("QEMU hit start breakpoint");
                    let pc: GuestReg = qemu
                        .read_reg(Regs::Pc)
                        .map_err(|e| Error::unknown(format!("Failed to read PC: {e:?}")))?;
                    log::info!("PC = {pc:#x}");
                },
                _ => panic!("Unexpected QEMU exit."),
            }
        }
        qemu.remove_breakpoint(start_pc);

        log::info!("Num Regs: {}", qemu.num_regs());
        log::info!("Now LibAFL takes control");
        
        // qemu.run() will run the emulator until the next breakpoint / sync exit, or until finish.
        qemu.set_breakpoint(end_pc);

        let input_addr = qemu
            .map_private(0, MAX_INPUT_SIZE, MmapPerms::ReadWrite)
            .map_err(|e| Error::unknown(format!("Failed to map input buffer: {e:}")))?;

        let pc: GuestReg = qemu
            .read_reg(Regs::Pc)
            .map_err(|e| Error::unknown(format!("Failed to read PC: {e:?}")))?;

        let stack_ptr: GuestAddr = qemu
            .read_reg(Regs::Sp)
            .map_err(|e| Error::unknown(format!("Failed to read stack pointer: {e:?}")))?;

        let ret_addr: GuestAddr = qemu
            .read_return_address()
            .map_err(|e| Error::unknown(format!("Failed to read return address: {e:?}")))?;

        Ok(Harness {
            qemu,
            input_addr,
            pc,
            stack_ptr,
            ret_addr,
        })
    }

    /// If we need to do extra work after forking, we can do that here.
    #[inline]
    #[expect(clippy::unused_self)]
    pub fn post_fork(&self) {}

    pub fn run(&self, input: &BytesInput) -> ExitKind {
        log::info!("Harness Start running");

        self.reset(input).unwrap();
        ExitKind::Ok
    }

    fn reset(&self, input: &BytesInput) -> Result<(), Error> {
        let target = input.target_bytes();
        let mut buf = target.as_slice();
        let mut len = buf.len();
        if len > MAX_INPUT_SIZE {
            buf = &buf[0..MAX_INPUT_SIZE];
            len = MAX_INPUT_SIZE;
        }
        let len = len as GuestReg;

        self.qemu.write_mem(self.input_addr, buf).map_err(|e| {
            Error::unknown(format!(
                "Failed to write to memory@{:#x}: {e:?}",
                self.input_addr
            ))
        })?;

        self.qemu
            .write_reg(Regs::Pc, self.pc)
            .map_err(|e| Error::unknown(format!("Failed to write PC: {e:?}")))?;

        self.qemu
            .write_reg(Regs::Sp, self.stack_ptr)
            .map_err(|e| Error::unknown(format!("Failed to write SP: {e:?}")))?;

        self.qemu
            .write_return_address(self.ret_addr)
            .map_err(|e| Error::unknown(format!("Failed to write return address: {e:?}")))?;

        self.qemu
            .write_function_argument(CallingConvention::Cdecl, 0, self.input_addr)
            .map_err(|e| Error::unknown(format!("Failed to write argument 0: {e:?}")))?;

        self.qemu
            .write_function_argument(CallingConvention::Cdecl, 1, len)
            .map_err(|e| Error::unknown(format!("Failed to write argument 1: {e:?}")))?;
        unsafe {
            let _ = self.qemu.run();
        };
        Ok(())
    }
}
