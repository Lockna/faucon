use faucon_asm::instruction::InstructionKind;
use faucon_asm::{Instruction, Operand};

use crate::cpu::{Cpu, CpuFlag};

/// Macro to extract all required operands out of a instruction and
/// reads a value from them using `read_value` or just returns the `Operand`.
macro_rules! operands {
    ($cpu:ident, $inst:ident, $($val:ident)|*) => {{
        let mut operands = $inst.operands().unwrap();
        ($(
            operands!(@internal, operands.pop().unwrap(), $cpu, $val)
        ),*)
    }};

    (@internal, $op:expr, $cpu:ident, val) => {
        read_value($op, $cpu)
    };

    (@internal, $op:expr, $cpu:ident, op) => {
        $op
    };
}

/// Macro to expand an operand to a given expression when it is guaranteed that
/// it takes a specific variant.
///
/// TODO: Remove this macro. This macro should be removed, but it can't atm, because
/// the `$flags` Operand still requires special handling.
macro_rules! operand {
    ($operand:expr, $($variant:pat)|* => $result:expr) => {
        if false {
            unreachable!()
        } $(else if let $variant = $operand {
            Some($result)
        })* else {
            None
        }
    };
}

/// Checks if the sign bit of an instruction result is set.
///
/// This is necessary to determine whether the sign flag
/// should be set for ALU instructions.
fn is_sign(x: u32, insn: &Instruction) -> bool {
    let sz: u32 = insn.operand_size().into();

    (x >> (sz - 1) & 1) != 0
}

/// Checks the high bits of 2 operands and their result to determine
/// whether there is a carry out.
///
/// This is necessary to determine whether the carry flag
/// should be set for ALU instructions.
fn is_carry(a: bool, b: bool, c: bool) -> bool {
    if a && b {
        // If a and b are both set, there is always a carry out.
        true
    } else if (a || b) && !c {
        // If either a or b are set and result is not, there is a carry.
        true
    } else {
        // Otherwise, there is no possibility of a carry.
        false
    }
}

/// Checks the high bits of 2 operands and their result to determine
/// whether there is a signed overflow.
///
/// This is necessary to determine whether the overflow flag
/// should be set for ALU instructions.
fn is_overflow(a: bool, b: bool, c: bool) -> bool {
    a == b && a != c
}

/// Emulates a given CPU instruction.
///
/// Returns the amount of CPU cycles that the instruction took.
pub fn process_instruction(cpu: &mut Cpu, insn: &Instruction) -> usize {
    match insn.kind {
        InstructionKind::ADD(_, _) => add(cpu, insn),
        InstructionKind::ADC(_, _) => adc(cpu, insn),
        InstructionKind::SUB(_, _) => sub(cpu, insn),
        InstructionKind::SBB(_, _) => sbb(cpu, insn),
        InstructionKind::AND(_, _) => and(cpu, insn),
        InstructionKind::OR(_, _) => or(cpu, insn),
        InstructionKind::XOR(_, _) => xor(cpu, insn),
        InstructionKind::XBIT(_, _) => xbit(cpu, insn),
        InstructionKind::BSET(_, _) => bset(cpu, insn),
        InstructionKind::BCLR(_, _) => bclr(cpu, insn),
        InstructionKind::BTGL(_, _) => btgl(cpu, insn),
        InstructionKind::SETP(_, _) => setp(cpu, insn),
        _ => todo!("Emulate remaining instructions"),
    }
}

/// Executes an ADD instruction.
fn add(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 + source2;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(
        CpuFlag::CARRY,
        is_carry(
            is_sign(source1, insn),
            is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers.set_flag(
        CpuFlag::OVERFLOW,
        is_overflow(
            is_sign(source1, insn),
            is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers
        .set_flag(CpuFlag::NEGATIVE, is_sign(result, insn));
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes an ADC instruction.
fn adc(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 + source2 + cpu.registers.get_flag(CpuFlag::CARRY) as u32;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(
        CpuFlag::CARRY,
        is_carry(
            is_sign(source1, insn),
            is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers.set_flag(
        CpuFlag::OVERFLOW,
        is_overflow(
            is_sign(source1, insn),
            is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers
        .set_flag(CpuFlag::NEGATIVE, is_sign(result, insn));
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes a SUB instruction.
fn sub(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 - source2;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(
        CpuFlag::CARRY,
        !is_carry(
            is_sign(source1, insn),
            !is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers.set_flag(
        CpuFlag::OVERFLOW,
        is_overflow(
            is_sign(source1, insn),
            !is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers
        .set_flag(CpuFlag::NEGATIVE, is_sign(result, insn));
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes an SBB instruction.
fn sbb(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 - source2 - cpu.registers.get_flag(CpuFlag::CARRY) as u32;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(
        CpuFlag::CARRY,
        !is_carry(
            is_sign(source1, insn),
            !is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers.set_flag(
        CpuFlag::OVERFLOW,
        is_overflow(
            is_sign(source1, insn),
            !is_sign(source2, insn),
            is_sign(result, insn),
        ),
    );
    cpu.registers
        .set_flag(CpuFlag::NEGATIVE, is_sign(result, insn));
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes an AND instruction.
fn and(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 & source2;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(CpuFlag::CARRY, false);
    cpu.registers.set_flag(CpuFlag::OVERFLOW, false);
    cpu.registers
        .set_flag(CpuFlag::NEGATIVE, is_sign(result, insn));
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes an OR instruction.
fn or(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 | source2;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(CpuFlag::CARRY, false);
    cpu.registers.set_flag(CpuFlag::OVERFLOW, false);
    cpu.registers
        .set_flag(CpuFlag::NEGATIVE, is_sign(result, insn));
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes an XOR instruction.
fn xor(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 ^ source2;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(CpuFlag::CARRY, false);
    cpu.registers.set_flag(CpuFlag::OVERFLOW, false);
    cpu.registers
        .set_flag(CpuFlag::NEGATIVE, is_sign(result, insn));
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes an XBIT instruction.
fn xbit(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (destination, source1, source2) = operands!(cpu, insn, op | val | val);

    // Compute the result of the operation and store it.
    let result = source1 >> source2 & 1;
    write_value(destination, cpu, result);

    // Set the CPU flags accordingly.
    cpu.registers.set_flag(CpuFlag::NEGATIVE, false);
    cpu.registers.set_flag(CpuFlag::ZERO, result == 0);

    1
}

/// Executes a BSET instruction.
fn bset(cpu: &mut Cpu, insn: &Instruction) -> usize {
    let operands = insn.operands().unwrap();

    // Extract the operands required to perform the operation.
    let destination = match insn.opcode() {
        0xF0 | 0xFD => operand!(operands[0], Operand::Register(reg) => reg),
        0xF4 | 0xF9 => None,
        _ => unreachable!(),
    };
    let source = match insn.opcode() {
        0xF0 => operand!(operands[1], Operand::I8(int) => int as u32).unwrap(),
        0xFD => {
            operand!(operands[1], Operand::Register(reg) => cpu.registers.get_gpr(reg)).unwrap()
        }
        0xF4 => operand!(operands[0], Operand::I8(int) => int as u32).unwrap(),
        0xF9 => {
            operand!(operands[0], Operand::Register(reg) => cpu.registers.get_gpr(reg)).unwrap()
        }
        _ => unreachable!(),
    };

    // Compute the result of the operation and store it.
    let bit = 1 << (source & 0x1F);
    if let Some(reg) = destination {
        let result = cpu.registers.get_gpr(reg) | bit;
        cpu.registers.set_gpr(reg, result);
    } else {
        let result = cpu.registers.get_flags() | bit;
        cpu.registers.set_flags(result);
    }

    1
}

/// Executes a BCLR instruction.
fn bclr(cpu: &mut Cpu, insn: &Instruction) -> usize {
    let operands = insn.operands().unwrap();

    // Extract the operands required to perform the operation.
    let destination = match insn.opcode() {
        0xF0 | 0xFD => operand!(operands[0], Operand::Register(reg) => reg),
        0xF4 | 0xF9 => None,
        _ => unreachable!(),
    };
    let source = match insn.opcode() {
        0xF0 => operand!(operands[1], Operand::I8(int) => int as u32).unwrap(),
        0xFD => {
            operand!(operands[1], Operand::Register(reg) => cpu.registers.get_gpr(reg)).unwrap()
        }
        0xF4 => operand!(operands[0], Operand::I8(int) => int as u32).unwrap(),
        0xF9 => {
            operand!(operands[0], Operand::Register(reg) => cpu.registers.get_gpr(reg)).unwrap()
        }
        _ => unreachable!(),
    };

    // Compute the result of the operation and store it.
    let bit = 1 << (source & 0x1F);
    if let Some(reg) = destination {
        let result = cpu.registers.get_gpr(reg) & !bit;
        cpu.registers.set_gpr(reg, result);
    } else {
        let result = cpu.registers.get_flags() & !bit;
        cpu.registers.set_flags(result);
    }

    1
}

/// Executes a BTGL instruction.
fn btgl(cpu: &mut Cpu, insn: &Instruction) -> usize {
    let operands = insn.operands().unwrap();

    // Extract the operands required to perform the operation.
    let destination = match insn.opcode() {
        0xF0 | 0xFD => operand!(operands[0], Operand::Register(reg) => reg),
        0xF4 | 0xF9 => None,
        _ => unreachable!(),
    };
    let source = match insn.opcode() {
        0xF0 => operand!(operands[1], Operand::I8(int) => int as u32).unwrap(),
        0xFD => {
            operand!(operands[1], Operand::Register(reg) => cpu.registers.get_gpr(reg)).unwrap()
        }
        0xF4 => operand!(operands[0], Operand::I8(int) => int as u32).unwrap(),
        0xF9 => {
            operand!(operands[0], Operand::Register(reg) => cpu.registers.get_gpr(reg)).unwrap()
        }
        _ => unreachable!(),
    };

    // Compute the result of the operation and store it.
    let bit = 1 << (source & 0x1F);
    if let Some(reg) = destination {
        let result = cpu.registers.get_gpr(reg) ^ bit;
        cpu.registers.set_gpr(reg, result);
    } else {
        let result = cpu.registers.get_flags() ^ bit;
        cpu.registers.set_flags(result);
    }

    1
}

/// Executes a SETP instruction.
fn setp(cpu: &mut Cpu, insn: &Instruction) -> usize {
    // Extract the operands required to perform the operation.
    let (source1, source2) = operands!(cpu, insn, val | val);

    // Compute the result of the operation and store it.
    let bit = source2 & 0x1F;
    let result = (cpu.registers.get_flags() & !(1 << bit)) | (source1 & 1) << bit;
    cpu.registers.set_flags(result);

    1
}

fn read_value(operand: Operand, cpu: &Cpu) -> u32 {
    match operand {
        Operand::Register(reg) => cpu.registers.get_gpr(reg),
        Operand::I8(x) => x as u32,
        Operand::I16(x) => x as u32,
        Operand::I24(x) => x,
        Operand::I32(x) => x,
    }
}

fn write_value(operand: Operand, cpu: &mut Cpu, val: u32) {
    if let Operand::Register(reg) = operand {
        cpu.registers.set_gpr(reg, val);
    }
}
