use std::fmt::{Display, Formatter, Result};
use std::io::BufRead;

type Pair = (String, Inst);

fn main() {
    let stdin = std::io::stdin();
    let lock = stdin.lock();

    let insts = lock
        .lines()
        .map(|l| {
            let s = l.ok()?;
            Some((s.clone(), convert(&s)?))
        })
        .collect::<Option<Vec<Pair>>>()
        .unwrap();

    let mut n = 0;
    while n < insts.len() {
        let r = to_asm(&insts[n], insts.get(n + 1));
        println!("{}", r.0);
        n += r.1;
    }
}

fn to_asm(cur: &Pair, next: Option<&Pair>) -> (String, usize) {
    let ref line = cur.0;
    let ref inst = cur.1;
    let mut n = 1;

    let s = match inst.op {
        Op::RET => {
            if matches!(inst.from, Reg::MI(6)) && matches!(inst.to, Reg::D(7)) {
                format!("RET\t\t\t; {}", line)
            } else {
                format!("RET?({:X})\t\t; {}", inst.orig, line)
            }
        }
        Op::HLT => {
            if matches!(inst.from, Reg::D(_)) && matches!(inst.to, Reg::D(_)) {
                format!("HLT\t\t\t; {}", line)
            } else {
                format!("HLT?({:X})\t\t; {}", inst.orig, line)
            }
        }
        Op::CLR => {
            if matches!(inst.from, Reg::D(_)) {
                format!("CLR\t{}\t\t; {}", inst.to, line)
            } else {
                format!("CLR?({:X})\t\t\t; {}", inst.orig, line)
            }
        }
        Op::JMP | Op::BRZ | Op::BRN => {
            if matches!(inst.to, Reg::D(7)) {
                match inst.from {
                    Reg::IP(7) => {
                        let ref next_line = next.unwrap().0;
                        let ref next_inst = next.unwrap().1;
                        n += 1;
                        format!(
                            "{}\t#0x{:04X}\t; {} & {}",
                            inst.op, next_inst.orig, line, next_line
                        )
                    }
                    _ => format!("{}\t{}\t\t; {}", inst.op, inst.from, line),
                }
            } else {
                format!("{}?({:X})\t\t\t; {}", inst.op, inst.orig, line)
            }
        }
        _ => match inst.from {
            Reg::IP(7) => {
                let ref next_line = next.unwrap().0;
                let ref next_inst = next.unwrap().1;
                n += 1;
                format!(
                    "{}\t#0x{:04X}, {}\t; {} & {}",
                    inst.op, next_inst.orig, inst.to, line, next_line
                )
            }
            _ => format!("{}\t; {}", inst, line),
        },
    };
    (s, n)
}

fn convert(l: &str) -> Option<Inst> {
    let mut sp = l.split(':');
    let inst_bin = u16::from_str_radix(sp.nth(1)?, 16).ok()?;

    Some(Inst::new(inst_bin))
}

struct Inst {
    op: Op,
    from: Reg,
    to: Reg,
    orig: u16,
}

impl Inst {
    fn new(bin: u16) -> Inst {
        let op = ((bin & 0b1111_11_00000_00000) >> 10) as u8;
        let from = ((bin & 0b0000_00_11111_00000) >> 5) as u8;
        let to = (bin & 0b0000_00_00000_11111) as u8;
        Inst {
            op: Op::from(op),
            from: Reg::from(from),
            to: Reg::from(to),
            orig: bin,
        }
    }
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}\t{}, {}", self.op, self.from, self.to)?;
        Ok(())
    }
}

enum Op {
    CLR,
    MOV,
    JMP,
    RET,
    ADD,
    SUB,
    CMP,
    JSR,
    BRN,
    BRZ,
    HLT,
    NOP,
    UNK(u8),
}

impl From<u8> for Op {
    fn from(op_bin: u8) -> Op {
        match op_bin {
            0b0001_00 => Op::CLR,
            0b0100_00 => Op::MOV,
            0b0100_01 => Op::JMP,
            0b0100_10 => Op::RET,
            0b0101_00 => Op::ADD,
            0b0110_00 => Op::SUB,
            0b0110_11 => Op::CMP,
            0b1011_00 => Op::JSR,
            0b1100_00 => Op::BRN,
            0b1100_01 => Op::BRZ,
            _ => match (op_bin & 0b1111_00) >> 2 {
                0b0000 => Op::HLT,
                0b0111 => Op::NOP,
                _ => Op::UNK(op_bin),
            },
        }
    }
}

impl Display for Op {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            Op::CLR => write!(fmt, "CLR")?,
            Op::MOV => write!(fmt, "MOV")?,
            Op::JMP => write!(fmt, "JMP")?,
            Op::RET => write!(fmt, "RET")?,
            Op::ADD => write!(fmt, "ADD")?,
            Op::SUB => write!(fmt, "SUB")?,
            Op::CMP => write!(fmt, "CMP")?,
            Op::HLT => write!(fmt, "HLT")?,
            Op::NOP => write!(fmt, "NOP")?,
            Op::JSR => write!(fmt, "JSR")?,
            Op::BRN => write!(fmt, "BRN")?,
            Op::BRZ => write!(fmt, "BRZ")?,
            Op::UNK(bin) => write!(fmt, "UNK({:06b})", bin)?,
        }
        Ok(())
    }
}

enum Reg {
    D(u8),
    I(u8),
    MI(u8),
    IP(u8),
    UNK(u8),
}

impl From<u8> for Reg {
    fn from(reg_bin: u8) -> Reg {
        let addr = (reg_bin & 0b11_000) >> 3;
        let reg = reg_bin & 0b00_111;
        match addr {
            0b00 => Reg::D(reg),
            0b01 => Reg::I(reg),
            0b10 => Reg::MI(reg),
            0b11 => Reg::IP(reg),
            _ => Reg::UNK(reg_bin),
        }
    }
}

impl Display for Reg {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            Reg::D(r) => write!(fmt, "R{}", r)?,
            Reg::I(r) => write!(fmt, "(R{})", r)?,
            Reg::MI(r) => write!(fmt, "-(R{})", r)?,
            Reg::IP(r) => write!(fmt, "(R{})+", r)?,
            Reg::UNK(bin) => write!(fmt, "UNK({:05b})", bin)?,
        }
        Ok(())
    }
}
