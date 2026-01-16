use std::env;
use std::fmt::Write;
use std::fs;

const REGISTER_MAP: [[&str; 2]; 8] = [
    ["al", "ax"],
    ["cl", "cx"],
    ["dl", "dx"],
    ["bl", "bx"],
    ["ah", "sp"],
    ["ch", "bp"],
    ["dh", "si"],
    ["bh", "di"],
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() != 2 {
        panic!("Usage: ./sim8086 path/to/binary/file");
    } else {
        args[1].parse::<String>().unwrap()
    };

    let instruction_stream = fs::read(file_path).expect("Could not read file");

    let mut arg1 = String::with_capacity(128);
    let mut arg2 = String::with_capacity(128);
    decode_instructions(&instruction_stream, &mut arg1, &mut arg2);
}

fn decode_instructions(mut bytes: &[u8], arg1: &mut String, arg2: &mut String) {
    // The while loop is needed because different instructions have different lengths
    'decode: while !bytes.is_empty() {
        // Clear the arena like strings
        arg1.clear();
        arg2.clear();

        let byte1 = bytes[0];

        // Match 4 bit instructions
        let opcode = byte1 >> 4;
        match opcode {
            0b1011 => {
                decode_mov_imm_reg(&mut bytes);
                continue 'decode;
            }
            _ => {}
        };

        // Match 6 bit instructions
        let opcode = byte1 >> 2;
        match opcode {
            0b100010 => {
                decode_mov_regmem_reg(&mut bytes, arg1, arg2);
                continue 'decode;
            }
            _ => {}
        };

        // Match 7 bit instructions
        let opcode = byte1 >> 1;
        match opcode {
            0b1010000 => {
                decode_mov_mem_accu(&mut bytes, true);
                continue 'decode;
            }
            0b1010001 => {
                decode_mov_mem_accu(&mut bytes, false);
                continue 'decode;
            }
            _ => panic!("Unsupported instruction. Opcode byte: {byte1:b}"),
        }
    }
}

fn decode_mov_regmem_reg(bytes: &mut &[u8], arg1: &mut String, arg2: &mut String) {
    const D_BIT_SHIFT: u8 = 1;
    const D_BIT_MASK: u8 = 0b00000010;
    const W_BIT_SHIFT: u8 = 0;
    const W_BIT_MASK: u8 = 0b00000001;
    const MOD_SHIFT: u8 = 6;
    const REG_SHIFT: u8 = 3;
    const REG_MASK: u8 = 0b00111000;
    const RM_MASK: u8 = 0b000000111;

    let byte1 = bytes[0];
    *bytes = &bytes[1..];

    let w_bit = ((byte1 & W_BIT_MASK) >> W_BIT_SHIFT) as usize;
    let d_bit: bool = matches!((byte1 & D_BIT_MASK) >> D_BIT_SHIFT, 1);

    if bytes.is_empty() {
        panic!("Not enough bytes to decode instructions");
    }
    let byte2 = bytes[0];
    *bytes = &bytes[1..];
    let mod_bytes = byte2 >> MOD_SHIFT;

    let reg = ((byte2 & REG_MASK) >> REG_SHIFT) as usize;
    let reg_arg: &str = REGISTER_MAP[reg][w_bit];
    let r_m = (byte2 & RM_MASK) as usize;

    let (dst, src) = match d_bit {
        true => (&mut *arg1, &mut *arg2),
        false => (&mut *arg2, &mut *arg1),
    };
    if mod_bytes == 0b11 {
        let rm_arg: &str = REGISTER_MAP[r_m][w_bit];

        dst.push_str(reg_arg);
        src.push_str(rm_arg);
    } else {
        let reg = match r_m {
            0b000 => "bx + si",
            0b001 => "bx + di",
            0b010 => "bp + si",
            0b011 => "bp + di",
            0b100 => "si",
            0b101 => "di",
            0b110 => "bp",
            0b111 => "bx",
            _ => panic!("Invalid R/M"),
        };

        // Direct address mode
        if r_m == 0b110 && mod_bytes == 0 {
            let address: u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
            *bytes = &bytes[2..];

            write!(src, "[{address}]").unwrap();
            write!(dst, "{reg}").unwrap();
        } else {
            let mut displacement: i16 = 0;
            if mod_bytes == 0b01 {
                displacement = (bytes[0] as i8) as i16;
                *bytes = &bytes[1..];
            } else if mod_bytes == 0b10 {
                displacement = i16::from_le_bytes([bytes[0], bytes[1]]);
                *bytes = &bytes[2..];
            }
            write!(dst, "{}", reg_arg).unwrap();
            if displacement != 0 {
                write!(src, "[{} + {}]", reg, displacement).unwrap();
            } else {
                write!(src, "[{}]", reg).unwrap();
            }
        }
    }

    println!("mov {}, {}", arg1, arg2);
}

fn decode_mov_imm_reg(bytes: &mut &[u8]) {
    const W_BIT_MASK: u8 = 0b00001000;
    const W_BIT_SHIFT: u8 = 3;
    const REG_MASK: u8 = 0b00000111;

    let byte1 = bytes[0];
    *bytes = &bytes[1..];
    let w_bit = ((byte1 & W_BIT_MASK) >> W_BIT_SHIFT) as usize;
    let reg = (byte1 & REG_MASK) as usize;
    let reg_arg: &str = REGISTER_MAP[reg][w_bit];

    let mut immediate: i16 = 0;
    if w_bit == 1 {
        immediate = i16::from_le_bytes([bytes[0], bytes[1]]);
        *bytes = &bytes[2..];
    } else {
        immediate |= (bytes[0] as i8) as i16;
        *bytes = &bytes[1..];
    }

    println!("mov {}, {}", reg_arg, immediate);
}

fn decode_mov_mem_accu(bytes: &mut &[u8], accu_first: bool) {
    let address = u16::from_le_bytes([bytes[1], bytes[2]]);
    *bytes = &bytes[3..];

    if accu_first {
        println!("mov ax, [{address}]");
    } else {
        println!("mov [{address}], ax");
    }
}
