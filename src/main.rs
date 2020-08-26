#![allow(unused)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate nom;

#[macro_use]
mod macros;
mod code;
mod config;
mod debugger;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use clap::App;
use color_eyre::{
    eyre::{eyre, WrapErr},
    Result, Section,
};

use config::Config;
use debugger::Debugger;
use faucon_emu::cpu::Cpu;

const CONFIG_ENV: &str = "FAUCON_CONFIG";

fn read_config<P: AsRef<Path>>(config: Option<P>) -> Result<Config> {
    // Check for the config CLI argument.
    if let Some(path) = config {
        return Ok(Config::load(&path)?);
    }

    // Check for the FAUCON_CONFIG environment variable.
    if let Ok(path) = env::var(CONFIG_ENV) {
        return Ok(Config::load(&path)?);
    }

    Err(eyre!("no config provided")).with_suggestion(|| {
        format!(
            "provide a config via the -c flag or the {} environment variable",
            CONFIG_ENV
        )
    })
}

fn run_emulator<P: AsRef<Path>>(bin: P, config: Config, vi_mode: bool) -> Result<()> {
    // Prepare the CPU and load the supplied binary into IMEM.
    let mut cpu = Cpu::new();
    if let Err(()) = code::upload_to_imem(&mut cpu, 0, 0, &code::read_falcon_binary(bin)?) {
        return Err(eyre!("the binary file is too large"))
            .wrap_err("failed to upload code")
            .with_suggestion(|| {
                format!(
                    "load a binary that is smaller than {} bytes \
                    or increase the IMEM size in the config",
                    config.falcon.get_imem_size()
                )
            });
    }

    // Create the debugger and run the REPL until the user exits.
    let mut debugger = Debugger::new(cpu, vi_mode);
    debugger.run().wrap_err("error in debugger repl occurred")?;

    Ok(())
}

fn disassemble_file<P: AsRef<Path>>(bin: P) -> Result<()> {
    let file = File::open(bin)?;
    let mut reader = BufReader::new(file);
    let insns = std::iter::from_fn(|| {
        use faucon_asm::Error;

        let insn = faucon_asm::read_instruction(&mut reader);
        match insn {
            Ok(insn) => Some(Ok(insn)),
            Err(Error::UnknownInstruction(op)) => {
                Some(Err(eyre!("encountered unknown instruction {:x}", op)))
            }
            Err(Error::IoError) => Some(Err(eyre!("unknown i/o error occurred"))),
            Err(Error::Eof) => None,
        }
    })
    .collect::<Result<Vec<_>>>()?;

    faucon_asm::pretty_print(insns.as_ref())?;
    Ok(())
}

fn get_binary_file<'matches>(
    matches: &'matches clap::ArgMatches<'matches>,
) -> Result<&'matches str> {
    if let Some(bin) = matches.value_of("binary") {
        Ok(bin)
    } else {
        return Err(eyre!("no binary file to run provided"))
            .suggestion("provide a binary file using the -b argument");
    }
}

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .panic_note(
            "Consider reporting the bug on github (https://github.com/vbe0201/faucon/issues)",
        )
        .install()?;

    // Build the CLI.
    let cli = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli).get_matches();

    // Read the configuration file.
    let config = read_config(matches.value_of("config")).wrap_err("failed to load config")?;

    if let Some(matches) = matches.subcommand_matches("emu") {
        run_emulator(
            get_binary_file(matches)?,
            config,
            matches.is_present("vi-mode"),
        )
    } else if let Some(matches) = matches.subcommand_matches("dis") {
        disassemble_file(get_binary_file(matches)?)
    } else {
        unreachable!()
    }
}
