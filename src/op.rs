#[derive(Debug)]
pub enum Op {
    Br,
    Add,
    Ld,
    St,
    Jsr,
    And,
    Ldr,
    Str,
    Rti,
    Not,
    Ldi,
    Sti,
    Jmp,
    Res,
    Lea,
    Trap,
}

impl TryFrom<u16> for Op {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, <Self as TryFrom<u16>>::Error> {
        match v {
            v if v == Op::Br as u16 => Ok(Op::Br),
            v if v == Op::Add as u16 => Ok(Op::Add),
            v if v == Op::Ld as u16 => Ok(Op::Ld),
            v if v == Op::St as u16 => Ok(Op::St),
            v if v == Op::Jsr as u16 => Ok(Op::Jsr),
            v if v == Op::And as u16 => Ok(Op::And),
            v if v == Op::Ldr as u16 => Ok(Op::Ldr),
            v if v == Op::Str as u16 => Ok(Op::Str),
            v if v == Op::Rti as u16 => Ok(Op::Rti),
            v if v == Op::Not as u16 => Ok(Op::Not),
            v if v == Op::Ldi as u16 => Ok(Op::Ldi),
            v if v == Op::Sti as u16 => Ok(Op::Sti),
            v if v == Op::Jmp as u16 => Ok(Op::Jmp),
            v if v == Op::Res as u16 => Ok(Op::Res),
            v if v == Op::Lea as u16 => Ok(Op::Lea),
            v if v == Op::Trap as u16 => Ok(Op::Trap),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Trap {
    /// get character from keyboard, not echoed onto the terminal
    Getc = 0x20,
    /// output a character
    Out = 0x21,
    /// output a word string
    Puts = 0x22,
    /// get character from keyboard, echoed onto the terminal
    In = 0x23,
    /// output a byte string
    Putsp = 0x24,
    /// halt the program
    Halt = 0x25,
}

impl TryFrom<u16> for Trap {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, <Self as TryFrom<u16>>::Error> {
        match v {
            v if v == Trap::Getc as u16 => Ok(Trap::Getc),
            v if v == Trap::Out as u16 => Ok(Trap::Out),
            v if v == Trap::Puts as u16 => Ok(Trap::Puts),
            v if v == Trap::In as u16 => Ok(Trap::In),
            v if v == Trap::Putsp as u16 => Ok(Trap::Putsp),
            v if v == Trap::Halt as u16 => Ok(Trap::Halt),
            _ => Err(()),
        }
    }
}
