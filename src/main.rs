use std::env;
use std::fs;

const OPCODE_SHIFT: u8 = 2;

const D_BIT_SHIFT: u8 = 1;
const D_BIT_MASK: u8 = 0b00000010;

const W_BIT_SHIFT: u8 = 0;
const W_BIT_MASK: u8 = 0b00000001;

const MOD_SHIFT: u8 = 6;

const REG_SHIFT: u8 = 3;
const REG_MASK: u8 = 0b00111000;

const RM_MASK: u8 = 0b000000111;

const MOV_OPCODE: u8 = 0b100010;

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
    decode_instructions(&instruction_stream);
}

fn decode_instructions(mut bytes: &[u8]) {
    // The while loop is needed because different instructions have different lengths
    while !bytes.is_empty() {
        let byte1 = bytes[0];
        bytes = &bytes[1..];

        let opcode = byte1 >> OPCODE_SHIFT;
        let inst = match opcode {
            MOV_OPCODE => "mov",
            _ => panic!("Unsupported instruction. Opcode: {opcode}"),
        };

        let w_bit = ((byte1 & W_BIT_MASK) >> W_BIT_SHIFT) as usize;
        let d_bit: bool = matches!((byte1 & D_BIT_MASK) >> D_BIT_SHIFT, 1);

        if bytes.is_empty() {
            panic!("Not enough bytes to decode instructions");
        }
        let byte2 = bytes[0];
        bytes = &bytes[1..];
        let mod_bytes = byte2 >> MOD_SHIFT;
        if mod_bytes != 0b11 {
            panic!("Cannot decode this mov instruction yet");
        }
        let reg = ((byte2 & REG_MASK) >> REG_SHIFT) as usize;
        let rm = (byte2 & RM_MASK) as usize;

        let reg_arg: &str = REGISTER_MAP[reg][w_bit];
        let rm_arg: &str = REGISTER_MAP[rm][w_bit];

        let (dest, src) = if d_bit {
            (reg_arg, rm_arg)
        } else {
            (rm_arg, reg_arg)
        };

        println!("{} {}, {}", inst, dest, src);
    }
}
