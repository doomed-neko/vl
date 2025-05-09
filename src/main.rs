#![allow(dead_code)]
use std::{
    env::args,
    fs::File,
    io::{Read, Write, stdin, stdout},
    process::ExitCode,
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum OpKind {
    Plus,
    Minus,
    Input,
    Output,
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
        Self::push_custom(ops, kind, 1);
    }
    fn push_custom(ops: &mut Vec<Op>, kind: OpKind, operand: u8) {
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
                Op::push_custom(&mut ops, OpKind::Input, 1);
            }
            '.' => {
                Op::push_custom(&mut ops, OpKind::Output, 1);
            }
            '<' => {
                Op::push(&mut ops, OpKind::Left);
            }
            '>' => {
                Op::push(&mut ops, OpKind::Right);
            }
            '[' => {
                bracket_stack.push(ops.len());
                Op::push_custom(&mut ops, OpKind::JumpZero, 0);
            }
            ']' => {
                let last_item = bracket_stack.pop();
                if let Some(id) = last_item {
                    ops[id].operand = ops.len() as u8 - 1;
                    Op::push_custom(&mut ops, OpKind::JumpNotZero, id as u8);
                } else {
                    eprintln!("ERROR: Unexpected ']' at index:{index}");
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
    if let Some(arg) = args.nth(1) {
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

    let mut stdout = stdout().lock();
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
            OpKind::Output => {
                let mut c = op.operand;
                let ch = stck[pntr as usize];
                while c > 0 {
                    let _ = stdout.write(&[ch]);
                    c -= 1;
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
            _ => (),
        }
    }

    ExitCode::SUCCESS
}
