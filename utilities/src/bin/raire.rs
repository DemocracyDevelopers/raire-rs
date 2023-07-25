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
use raire::RaireProblem;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// This is a Rust port of RAIRE, originally written by Michelle Blom and ported to Rust by Andrew Conway
struct CliOptions {
    /// The JSON file containing the command to RAIRE
    input_json_file : PathBuf,
    /// The file to store the output. Default is the input file name, with path and extension if present removed and `_out.json` added.
    output_json_file : Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = CliOptions::parse();
    let input : RaireProblem = serde_json::from_reader(File::open(&args.input_json_file)?)?;
    let output = input.solve();
    let output_file : PathBuf = args.output_json_file.unwrap_or_else(||{
        let mut stem = args.input_json_file.file_stem().map(|s|PathBuf::from(s)).unwrap_or_else(||PathBuf::from("output"));
        stem.as_mut_os_string().push("_out.json");
        stem
    });
    serde_json::to_writer(File::create(&output_file)?,&output)?;
    Ok(())
}