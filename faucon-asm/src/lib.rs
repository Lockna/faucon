//! Rust library for working with the NVIDIA Falcon ISA.
//!
//! # About the Falcon
//!
//! The **Fa**st **L**ogic **Con**troller is a series of general-purpose
//! embedded microprocessors that have been started in ~2005. With over
//! 3 billion units shipped, the Falcon has been used in many different
//! places, primarily NVIDIA GPUs starting from G98.
//!
//! Falcon units consist of:
//!
//! - the core processors with its SRAM for code and data
//! - the I/O space for communication with host systems
//! - FIFO interface logic (optional)
//! - memory interface logic (optional)
//! - cryptographic AES coprocessor, making the Falcon a "secretful unit" (optional)
//! - unit-specific logic, depending on how the processor is used (optional)
//!
//! # The Falcon Instruction Set Architecture
//!
//! This crate is providing abstract mechanisms for simplifying assembling/disassembling
//! Assembly instructions, while also providing a well-documented and understandable API
//! that explains the ISA concepts and how they are supposed to be handled.
//!
//! ## Disassembling
//!
//! ```
//! use std::io::Cursor;
//!
//! use faucon_asm::{disassemble, instruction::InstructionKind};
//!
//! let instructions = disassemble(&mut Cursor::new(vec![0xfa, 0x9b, 0x00])).unwrap();
//! assert_eq!(instructions.len(), 1);
//! assert_eq!(instructions[0].kind, InstructionKind::IOWR(0xfa, 0x0));
//! ```
//!
//! ## Assembling
//!
//! **TODO:** Implement this.

#![warn(missing_docs)]
#![allow(clippy::len_without_is_empty)]

use std::fmt;
use std::io::Read;

use byteorder::{ByteOrder, LittleEndian};

use instruction::*;
use operand::*;

pub mod disassembler;
pub mod instruction;
pub mod opcode;
pub mod operand;

/// The result of a Falcon ISA functions.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error types that are used by the crate.
///
/// These help in providing better diagnostics when using the crate
/// and to provide one uniform [`Result`] type within the crate.
///
/// [`Result`]: type.Result.html
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid instruction encountered.
    ///
    /// Provides the opcode of the instruction in question.
    InvalidInstruction(u8),
    /// An I/O error occurred while working with a reader.
    IoError,
    /// An EOF has been reached while streaming a binary's content through [`Read`].
    ///
    /// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    Eof,
}

/// A Falcon CPU register, encoded as an [`Instruction`] [`Operand`].
///
/// The Falcon has 16 general-purpose registers, indexed from 0 through 16
/// (denoted by the `value` field), along with around a dozen special registers.
///
/// [`Instruction`]: struct.Instruction.html
/// [`Operand`]: enum.Operand.html
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Register {
    /// The [`RegisterMeta`] object that corresponds to the wrapped register.
    ///
    /// Provides useful semantic details on its purpose within an [`Instruction`].
    ///
    /// [`Instruction`]: struct.Instruction.html
    pub meta: RegisterMeta,
    /// The value of the wrapped register.
    ///
    /// This is a number ranging from 0-15, denoting the index of the register.
    pub value: usize,
}

impl Register {
    /// Constructs a new register from its byte representation and its meta information.
    ///
    /// Registers are a kind of [`Operand`] that is being utilized in [`Instruction`]
    /// operations.
    ///
    /// [`Operand`]: enum.Operand.html
    /// [`Instruction`]: struct.Instruction.html
    pub fn new(register_meta: &RegisterMeta, insn: &[u8]) -> Self {
        let value = parse_register(insn, register_meta) as usize;
        Register {
            meta: *register_meta,
            value,
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "$r{}", self.value)
    }
}

/// A Falcon Assembly operand that belongs to an [`Instruction`].
///
/// Operands can either be a CPU [`Register`], or an immediate of a
/// variable size. A [`Vec`] of operands for a particular
/// [`Instruction`] can be obtained through [`Instruction::operands`].
/// It is within the user's responsibility to correctly determine
/// how the operands are meant to be interpreted and handled.
///
/// [`Instruction`]: struct.Instruction.html
/// [`Register`]: struct.Register.html
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`Instruction::operands`]: struct.Instruction.html#method.operands
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    /// A CPU [`Register`], wrapping around its value and the
    /// corresponding [`RegisterMeta`] object.
    ///
    /// [`Register`]: struct.Register.html
    /// [`RegisterMeta`]: ./operand/struct.RegisterMeta.html
    Register(Register),
    /// An 8-bit-sized immediate, represented through an [`u8`].
    ///
    /// [`u8`]: https://doc.rust-lang.org/stable/std/primitive.u8.html
    I8(u8),
    /// A 16-bit-sized immediate, represented through an [`u16`].
    ///
    /// [`u16`]: https://doc.rust-lang.org/stable/std/primitive.u16.html
    I16(u16),
    /// A 24-bit-sized immediate, represented through an [`u32`].
    ///
    /// [`u32`]: https://doc.rust-lang.org/stable/std/primitive.u32.html
    I24(u32),
    /// A 32-bit-sized immediate, represented through an [`u32`].
    ///
    /// [`u32`]: https://doc.rust-lang.org/stable/std/primitive.u32.html
    I32(u32),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::I8(int) => write!(f, "{:#02x}", int),
            Operand::I16(int) => write!(f, "{:#04x}", int),
            Operand::I24(int) => write!(f, "{:#06x}", int),
            Operand::I32(int) => write!(f, "{:#08x}", int),
        }
    }
}

/// A Falcon Assembly instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// The kind of instruction that is being wrapped.
    ///
    /// [`InstructionKind`]s generally identify instructions and
    /// their operands individually and provide helper methods for
    /// working with them.
    ///
    /// [`InstructionKind`]: ./instruction/enum.InstructionKind.html
    pub kind: InstructionKind,
    /// The raw bytes of an instruction.
    pub(crate) bytes: Vec<u8>,
}

impl Instruction {
    /// Constructs a new instruction from its kind and bytes representation.
    ///
    /// This function returns `None` if an invalid instruction was supplied.
    /// When the instruction was successfully created, its byte representation
    /// must be supplied through [`Instruction::feed`] before actually using any
    /// of the provided methods.
    ///
    /// [`Instruction::feed`]: struct.Instruction.html#method.feed
    pub fn new(kind: InstructionKind) -> Option<Self> {
        // Filter out invalid instructions.
        if kind.invalid() {
            return None;
        }

        Some(Instruction {
            kind,
            bytes: Vec::new(),
        })
    }

    /// Gets the length of an instruction by counting the amount of bytes that
    /// define it.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    fn parse_operand(&self, operand: &OperandMeta) -> Operand {
        match operand {
            OperandMeta::R(meta) => Operand::Register(Register::new(meta, &self.bytes)),
            OperandMeta::I8 => Operand::I8(self.bytes[2] as u8),
            OperandMeta::I16 => Operand::I16(LittleEndian::read_u16(&self.bytes[2..])),
            OperandMeta::I24 => Operand::I24(LittleEndian::read_u24(&self.bytes[2..])),
            OperandMeta::I32 => Operand::I32(LittleEndian::read_u32(&self.bytes[2..])),
        }
    }

    /// Gets the opcode of the instruction.
    ///
    /// The opcode is the first byte of each instruction, from which a lot
    /// of useful disassembling details can be derived. Together with a
    /// subopcode, instructions can be identified uniquely.
    pub fn opcode(&self) -> u8 {
        // This is considered safe, since only valid instructions
        // can be used for constructing this type.
        self.kind.opcode().unwrap()
    }

    /// Gets the subopcode of an instruction.
    ///
    /// The subopcode can be placed in various locations and is necessary
    /// for identifying instructions uniquely in combination with the
    /// opcode.
    pub fn subopcode(&self) -> u8 {
        // This is considered safe, since only valid instructions
        // can be used for constructing this type.
        self.kind.subopcode().unwrap()
    }

    /// A vector of [`Operand`]s that belong to this instruction.
    ///
    /// See [`Instruction::operand_size`] to determine the size of
    /// operands individually per instruction.
    ///
    /// [`Operand`]: enum.Operand.html
    /// [`Instruction::operand_size`]: struct.Instruction.html#method.operand_size
    pub fn operands(&self) -> Option<Vec<Operand>> {
        // Since there are instructions that might not take any
        // operands at all, it is better to return the Option
        // instead of unwrapping.
        if let Some(operands) = self.kind.operands() {
            Some(
                operands
                    .iter()
                    .map(|o| self.parse_operand(o))
                    .flat_map(|op| {
                        if let Operand::Register(Register {
                            meta: RegisterMeta(loc, RegisterDirection::SourceDestination),
                            value,
                        }) = op
                        {
                            let map = |meta: RegisterMeta, value: usize| {
                                Operand::Register(Register { meta, value })
                            };
                            let first = RegisterMeta(loc, RegisterDirection::Destination);
                            let second = RegisterMeta(loc, RegisterDirection::Source);
                            vec![map(first, value), map(second, value)].into_iter()
                        } else {
                            vec![op].into_iter()
                        }
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Gets the size of instruction operands.
    ///
    /// The size is derived from the first byte of an instruction.
    pub fn operand_size(&self) -> OperandSize {
        OperandSize::from(self.opcode())
    }

    /// Feeds a slice of bytes to the internal representation of the instruction.
    ///
    /// This method is supposed to be called right after a successful call to
    /// [`Instruction::new`] before actually using the object.
    ///
    /// [`Instruction::new`]: struct.Instruction.html#method.new
    pub fn feed(&mut self, bytes: &[u8]) {
        self.bytes.extend(bytes);
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // First, write the instruction mnemonic.
        write!(f, "{}", self.kind)?;

        // Then, check if the instruction has a specific
        // size mode notation. If so, print it as well.
        if self.operand_size() != OperandSize::Unsized {
            let sz: u32 = self.operand_size().into();
            write!(f, " b{}", sz)?;
        }

        // In special cases, the subopcode denotes small instruction details
        // that are relevant for the resulting disassembly output.
        if let InstructionKind::TRAP(_, subopcode) = self.kind {
            write!(f, " {}", subopcode - 0x8)?;
        }

        // Write the operands of the instruction, if any.
        if let Some(operands) = self.operands() {
            match self.kind {
                InstructionKind::IORD(_, _) => {
                    write!(
                        f,
                        " {} I[{} + {} * 4]",
                        operands[0], operands[1], operands[2]
                    )?;
                }
                InstructionKind::IOWR(0xD0, _) | InstructionKind::IOWRS(0xD0, _) => {
                    write!(
                        f,
                        " I[{} + {} * 4] {}",
                        operands[0], operands[1], operands[2]
                    )?;
                }
                InstructionKind::IOWR(0xFA, _)
                | InstructionKind::IOWR(0xF6, _)
                | InstructionKind::IOWRS(0xFA, _) => {
                    write!(f, " I[{}] {}", operands[0], operands[1])?;
                }
                InstructionKind::LD(0x10, _) | InstructionKind::LD(0x3C, _) => {
                    let sz: u32 = self.operand_size().into();

                    write!(
                        f,
                        " {} D[{} + {} * {}]",
                        operands[0],
                        operands[1],
                        operands[2],
                        sz / 8
                    )?;
                }
                InstructionKind::LD(0x34, _) | InstructionKind::LD(0x3A, _) => {
                    let sz: u32 = self.operand_size().into();

                    write!(f, " {} D[$sp + {} * {}]", operands[0], operands[1], sz / 8)?;
                }
                InstructionKind::ST(0x00, _) | InstructionKind::ST(0x38, 0) => {
                    let sz: u32 = self.operand_size().into();

                    write!(
                        f,
                        " D[{} + {} * {}] {}",
                        operands[0],
                        operands[1],
                        sz / 8,
                        operands[2]
                    )?;
                }
                InstructionKind::ST(0x30, _) | InstructionKind::ST(0x38, 1) => {
                    let sz: u32 = self.operand_size().into();

                    write!(f, " D[$sp + {} * {}] {}", operands[0], sz / 8, operands[1])?;
                }
                _ => {
                    for operand in operands.iter() {
                        write!(f, " {}", operand)?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Disassembles the bytes from a given stream until an EOF is encountered.
///
/// This function tries to disassemble as many bytes as provided through the
/// [`Read`]er, returning a [`Vec`] of [`Instruction`]s on EOF. As a logical
/// consequence, any [`Error`] variant returned by this instruction is never
/// an EOF.
///
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`Instruction`]: struct.Instruction.html
/// [`Error`]: enum.Error.html
pub fn disassemble<R: Read>(reader: &mut R) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();

    loop {
        match disassembler::read_instruction(reader) {
            Ok(insn) => instructions.push(insn),
            Err(e) => {
                return if e == Error::Eof {
                    Ok(instructions)
                } else {
                    Err(e)
                }
            }
        }
    }
}
