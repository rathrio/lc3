#[derive(Debug)]
pub enum Op {
    /// branch
    BR,
    /// add
    ADD,
    /// load
    LD,
    /// store
    ST,
    /// jump register
    JSR,
    /// bitwise and
    AND,
    /// load register
    LDR,
    /// store register
    STR,
    /// unused
    RTI,
    /// bitwise not
    NOT,
    /// load indirect
    LDI,
    /// store indirect
    STI,
    /// jump
    JMP,
    /// reserved (unused)
    RES,
    /// load effective address
    LEA,
    /// execute trap
    TRAP,
}

impl TryFrom<u16> for Op {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, <Self as TryFrom<u16>>::Error> {
        match v {
            v if v == Op::BR as u16 => Ok(Op::BR),
            v if v == Op::ADD as u16 => Ok(Op::ADD),
            v if v == Op::LD as u16 => Ok(Op::LD),
            v if v == Op::ST as u16 => Ok(Op::ST),
            v if v == Op::JSR as u16 => Ok(Op::JSR),
            v if v == Op::AND as u16 => Ok(Op::AND),
            v if v == Op::LDR as u16 => Ok(Op::LDR),
            v if v == Op::STR as u16 => Ok(Op::STR),
            v if v == Op::RTI as u16 => Ok(Op::RTI),
            v if v == Op::NOT as u16 => Ok(Op::NOT),
            v if v == Op::LDI as u16 => Ok(Op::LDI),
            v if v == Op::STI as u16 => Ok(Op::STI),
            v if v == Op::JMP as u16 => Ok(Op::JMP),
            v if v == Op::RES as u16 => Ok(Op::RES),
            v if v == Op::LEA as u16 => Ok(Op::LEA),
            v if v == Op::TRAP as u16 => Ok(Op::TRAP),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Trap {
    /// get character from keyboard, not echoed onto the terminal
    GETC = 0x20,
    /// output a character
    OUT = 0x21,
    /// output a word string
    PUTS = 0x22,
    /// get character from keyboard, echoed onto the terminal
    IN = 0x23,
    /// output a byte string
    PUTSP = 0x24,
    /// halt the program
    HALT = 0x25,
}

impl TryFrom<u16> for Trap {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, <Self as TryFrom<u16>>::Error> {
        match v {
            v if v == Trap::GETC as u16 => Ok(Trap::GETC),
            v if v == Trap::OUT as u16 => Ok(Trap::OUT),
            v if v == Trap::PUTS as u16 => Ok(Trap::PUTS),
            v if v == Trap::IN as u16 => Ok(Trap::IN),
            v if v == Trap::PUTSP as u16 => Ok(Trap::PUTSP),
            v if v == Trap::HALT as u16 => Ok(Trap::HALT),
            _ => Err(()),
        }
    }
}
