#[derive(PartialEq, Eq)]
pub enum InterruptType {
    Nmi,
    Brk,
}

#[derive(PartialEq, Eq)]
pub(crate) struct Interrupt {
    pub(super) itype: InterruptType,
    pub(super) vector_addr: u16,
    pub(super) b_flag_mask: u8,
    pub(super) cpu_cycles: u8,
}
pub(crate) const NMI: Interrupt = Interrupt {
    itype: InterruptType::Nmi,
    vector_addr: 0xfffA,
    b_flag_mask: 0b00100000,
    cpu_cycles: 2,
};

pub(super) const BRK: Interrupt = Interrupt {
    itype: InterruptType::Brk,
    vector_addr: 0xfffe,
    b_flag_mask: 0b00110000,
    cpu_cycles: 1,
};
