// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.



use std::fs::File;
use std::path::PathBuf;

use clap::{Parser};
use raire::irv::{CandidateIndex};
use raire::{RaireSolution};
use raire::assertions::Assertion;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// This reads the output of a RAIRE computation and produces a human readable description
struct CliOptions {
    /// The output from RAIRE
    input_file : PathBuf,
}


fn main() -> anyhow::Result<()> {
    let args = CliOptions::parse();
    let input : RaireSolution = serde_json::from_reader(File::open(&args.input_file)?)?;
    let name = |c:CandidateIndex| {
        if let Some(metadata_name) = input.metadata["candidates"][c.0 as usize].as_str() {
            metadata_name.to_string()
        } else {
            format!("#{}",c.0)
        }
    };
    match &input.solution {
        Ok(solution) => {
            println!("Solution overall difficulty {}",solution.difficulty);
            for a in &solution.assertions {
                match &a.assertion {
                    Assertion::NEB(neb) => print!("{} NEB {}",name(neb.winner),name(neb.loser)),
                    Assertion::NEN(nen) => print!("{} > {} with {:?} continuing",name(nen.winner),name(nen.loser),nen.continuing.iter().cloned().map(name).collect::<Vec<_>>()),
                }
                println!("  Difficulty {}",a.difficulty);
            }
        }
        Err(e) => {
            println!("Could not find a solution because {:?}",e)
        }
    }
    Ok(())
}