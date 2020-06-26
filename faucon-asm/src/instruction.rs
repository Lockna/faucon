//! Falcon Assembly instruction listings.

use faucon_asm_derive::Instruction;

use crate::operand::OperandMeta;

/// Assembly instructions that are supported by the Falcon ISA.
///
/// This enum is expanded by an internally used proc-macro which
/// simplifies parsing instructions, given an `(opcode, subopcode)`
/// pair. See the respective methods which ease up parsing work.
#[derive(Clone, Debug, PartialEq, Eq, Instruction)]
pub enum InstructionKind {
    /// The AND instruction.
    ///
    /// Applies a bitwise and operation on two operands and stores
    /// the result.
    #[insn(opcode = 0xC0, subopcode = 0x04, operands = "R1D, R2S, I8")]
    #[insn(opcode = 0xE0, subopcode = 0x04, operands = "R1D, R2S, I16")]
    #[insn(opcode = 0xF0, subopcode = 0x04, operands = "R2SD, I8")]
    #[insn(opcode = 0xF1, subopcode = 0x04, operands = "R2SD, I16")]
    #[insn(opcode = 0xFD, subopcode = 0x04, operands = "R2SD, R1S")]
    #[insn(opcode = 0xFF, subopcode = 0x04, operands = "R3D, R2S, R1S")]
    AND(u8, u8, String),

    /// The OR instruction.
    ///
    /// Applies a bitwise or operation on two operands and stores
    /// the result.
    #[insn(opcode = 0xC0, subopcode = 0x05, operands = "R1D, R2S, I8")]
    #[insn(opcode = 0xE0, subopcode = 0x05, operands = "R1D, R2S, I16")]
    #[insn(opcode = 0xF0, subopcode = 0x05, operands = "R2SD, I8")]
    #[insn(opcode = 0xF1, subopcode = 0x05, operands = "R2SD, I16")]
    #[insn(opcode = 0xFD, subopcode = 0x05, operands = "R2SD, R1S")]
    #[insn(opcode = 0xFF, subopcode = 0x05, operands = "R3D, R2S, R1S")]
    OR(u8, u8, String),

    /// The XOR instruction.
    ///
    /// Applies a bitwise xor operation on two operands and stores
    /// the result.
    #[insn(opcode = 0xC0, subopcode = 0x06, operands = "R1D, R2S, I8")]
    #[insn(opcode = 0xE0, subopcode = 0x06, operands = "R1D, R2S, I16")]
    #[insn(opcode = 0xF0, subopcode = 0x06, operands = "R2SD, I8")]
    #[insn(opcode = 0xF1, subopcode = 0x06, operands = "R2SD, I16")]
    #[insn(opcode = 0xFD, subopcode = 0x06, operands = "R2SD, R1S")]
    #[insn(opcode = 0xFF, subopcode = 0x06, operands = "R3D, R2S, R1S")]
    XOR(u8, u8, String),

    /// The XBIT instruction.
    ///
    /// Extracts a single bit of a specified register and stores it in the
    /// highest bit of the destination register, setting all other bits to 0.
    #[insn(opcode = 0xC0, subopcode = 0x08, operands = "R1D, R2S, I8")]
    #[insn(opcode = 0xFF, subopcode = 0x08, operands = "R3D, R2S, R1S")]
    #[insn(opcode = 0xF0, subopcode = 0x0C, operands = "R2D, I8")]
    #[insn(opcode = 0xFE, subopcode = 0x0C, operands = "R1D, R2S")]
    XBIT(u8, u8, String),

    /// The BSET instruction.
    ///
    /// Sets a specific bit in a given register.
    #[insn(opcode = 0xF0, subopcode = 0x09, operands = "R2D, I8")]
    #[insn(opcode = 0xFD, subopcode = 0x09, operands = "R2D, R1S")]
    #[insn(opcode = 0xF4, subopcode = 0x31, operands = "I8")]
    #[insn(opcode = 0xF9, subopcode = 0x09, operands = "R2S")]
    BSET(u8, u8, String),

    /// The BCLR instruction.
    ///
    /// Clears a specific bit in a given register.
    #[insn(opcode = 0xF0, subopcode = 0x0A, operands = "R2D, I8")]
    #[insn(opcode = 0xFD, subopcode = 0x0A, operands = "R2D, R1S")]
    #[insn(opcode = 0xF4, subopcode = 0x32, operands = "I8")]
    #[insn(opcode = 0xF9, subopcode = 0x0A, operands = "R2S")]
    BCLR(u8, u8, String),

    /// The BTGL instruction.
    ///
    /// Toggles (flips) a specific bit in a given register.
    #[insn(opcode = 0xF0, subopcode = 0x0B, operands = "R2D, I8")]
    #[insn(opcode = 0xFD, subopcode = 0x0B, operands = "R2D, R1S")]
    #[insn(opcode = 0xF4, subopcode = 0x33, operands = "I8")]
    #[insn(opcode = 0xF9, subopcode = 0x0B, operands = "R2S")]
    BTGL(u8, u8, String),

    /// The IOWR instruction.
    ///
    /// Writes a word to the I/O space of the processor.
    #[insn(opcode = 0xFA, subopcode = 0x0, operands = "R2S, R1S")]
    IOWR(u8, u8, String),

    /// An invalid or unknown instruction.
    ///
    /// This can be checked through calling [`InstructionKind::invalid`].
    /// If such an instruction is encountered, the user may halt the
    /// program.
    ///
    /// [`InstructionKind::invalid`]: enum.InstructionKind.html#method.invalid
    XXX,
}
