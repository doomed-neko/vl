#![allow(dead_code)]
use std::{
    env::args,
    fs::File,
    io::{stdin, Read},
    process::ExitCode,
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum OpKind {
    Plus,
    Minus,
    Input,
    OutputAscii,
    OutputNumber,
    Left,
    Right,
    JumpZero,
    JumpNotZero,
}

#[derive(Debug, Clone, Copy)]
struct Op {
    kind: OpKind,
    operand: u8,
}
impl Op {
    fn new(kind: OpKind, operand: u8) -> Self {
        Self { kind, operand }
    }
    fn push(ops: &mut Vec<Op>, kind: OpKind) {
        Self::push_custom_op(ops, kind, 1);
    }
    fn push_custom_op(ops: &mut Vec<Op>, kind: OpKind, operand: u8) {
        if let Some(op) = ops.last_mut() {
            if op.kind == kind {
                op.operand += 1;
                return;
            }
        }
        ops.push(Op::new(kind, operand));
    }
}

fn tokenize(mut file: File) -> Option<Vec<Op>> {
    let mut file_content = String::new();
    let mut ops = Vec::<Op>::new();
    file.read_to_string(&mut file_content)
        .expect("Unable to read file");

    let mut bracket_stack = Vec::<usize>::new();
    for (index, ch) in file_content.chars().enumerate() {
        match ch {
            '+' => {
                Op::push(&mut ops, OpKind::Plus);
            }
            '-' => {
                Op::push(&mut ops, OpKind::Minus);
            }
            ',' => {
                Op::push(&mut ops, OpKind::Input);
            }
            '.' => {
                Op::push(&mut ops, OpKind::OutputAscii);
            }
            '*' => {
                Op::push(&mut ops, OpKind::OutputNumber);
            }
            '<' => {
                Op::push(&mut ops, OpKind::Left);
            }
            '>' => {
                Op::push(&mut ops, OpKind::Right);
            }
            '[' => {
                bracket_stack.push(ops.len());
                Op::push(&mut ops, OpKind::JumpZero);
            }
            ']' => {
                let last_item = bracket_stack.pop();
                if let Some(id) = last_item {
                    ops[id].operand = ops.len() as u8 - 1;
                    Op::push_custom_op(&mut ops, OpKind::JumpNotZero, id as u8);
                } else {
                    eprintln!("ERROR: Unexpected ']' at index:{index}");
                    return None;
                }
            }
            _ => (),
        };
    }
    if bracket_stack.pop().is_some() {
        eprintln!("ERROR: Unclosed ']'");
        None
    } else {
        Some(ops)
    }
}

fn main() -> ExitCode {
    let mut args = args();
    let file_name: String;
    let program = args.next().unwrap();
    if let Some(arg) = args.next() {
        if arg == "--help" {
            println!("Usage:\n    {program} <filename>");
            return ExitCode::FAILURE;
        }
        file_name = arg;
    } else {
        eprintln!("ERROR: No file name provided\nUsage: vl <filename>");
        return ExitCode::FAILURE;
    }
    let file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: unable to open file:\n{e}");
            return ExitCode::FAILURE;
        }
    };
    let ops: Vec<Op>;
    if let Some(result) = tokenize(file) {
        ops = result
    } else {
        return ExitCode::FAILURE;
    }

    let mut stck: [u8; 30_000] = [0; 30_000];
    let mut pntr: u16 = 0;
    let mut index: usize = 0;
    while index < ops.len() {
        let op: &Op = &ops[index];
        index += 1;
        match op.kind {
            OpKind::Plus => {
                stck[pntr as usize] = stck[pntr as usize].wrapping_add(op.operand);
            }
            OpKind::Minus => {
                stck[pntr as usize] = stck[pntr as usize].wrapping_sub(op.operand);
            }
            OpKind::Input => {
                let _ = stdin().read_exact(&mut stck[pntr as usize..pntr as usize]);
            }
            OpKind::OutputAscii => {
                let ch = stck[pntr as usize];
                for _ in 0..op.operand {
                    print!("{}", ch as char);
                }
            }
            OpKind::OutputNumber => {
                let ch = stck[pntr as usize];
                for _ in 0..op.operand {
                    print!("{}", ch);
                }
            }
            OpKind::Right => {
                if pntr == 29_999 {
                    pntr = 0;
                } else {
                    pntr += op.operand as u16;
                }
            }
            OpKind::Left => {
                if pntr == 0 {
                    pntr = 29_999;
                } else {
                    pntr -= op.operand as u16;
                }
            }
            OpKind::JumpZero if stck[pntr as usize] == 0 => index = op.operand as usize,
            OpKind::JumpNotZero if stck[pntr as usize] != 0 => index = op.operand as usize,
            OpKind::JumpZero | OpKind::JumpNotZero => (),
        }
    }

    ExitCode::SUCCESS
}
