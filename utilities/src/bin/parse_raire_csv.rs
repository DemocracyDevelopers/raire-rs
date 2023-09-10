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
use anyhow::anyhow;

use clap::{Parser};
use raire::audit_type::{Audit, BallotComparisonMACRO, BallotComparisonOneOnDilutedMargin, BallotPollingBRAVO, BallotPollingOneOnDilutedMarginSquared};
use raire::irv::BallotPaperCount;
use utilities::parse_michelle_format::Contest;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// This reads the CSV files in the https://github.com/michelleblom/audit-irv-cp/tree/raire-branch repo and converts them to
/// the JSON unput for raire-rs
struct CliOptions {
    /// The CSV file containing the command to RAIRE
    input_raire_file : PathBuf,
    /// The file to store the output. Default is the input file name, with path and extension if present removed and `.json` added.
    /// If the contest is specified, there will be _(contest_index) added before the `.json`.
    output_json_file : Option<PathBuf>,
    /// If there are multiple contests in the input file, which one do you want, starting counting at 1 (default 1)
    #[arg(short, long)]
    contest : Option<usize>,
    /// set if you want ballot polling (default ballot comparison)
    #[arg(long)]
    ballot_polling : bool,
    /// the total number of ballots (if different from the number of votes in the file)
    #[arg(long)]
    total_ballots : Option<usize>,
    /// the desired confidence level (for MACRO or BRAVO). If not specified, then a 1/margin (or 1/margin squared) computation will be done.
    #[arg(long)]
    confidence : Option<f64>,
    /// the error_inflation_factor (for MACRO).
    #[arg(long)]
    error_inflation_factor : Option<f64>,
}

fn main() -> anyhow::Result<()> {
    let args = CliOptions::parse();
    let input : Vec<Contest> = Contest::parse(&args.input_raire_file)?;
    let output = {
        let index = args.contest.unwrap_or(1);
        if index<1 || index>input.len() { return Err(anyhow!("Contest number must be between 1 and {}",input.len()))}
        let contest = &input[index-1];
        let num_ballots : usize = contest.votes.values().sum();
        println!("{num_ballots} ballots of which {} are unique",contest.votes.len());
        let total_auditable_ballots = BallotPaperCount(args.total_ballots.unwrap_or(num_ballots));
        let audit : Audit = match (args.ballot_polling,args.confidence) {
            (false,None) => Audit::OneOnMargin(BallotComparisonOneOnDilutedMargin{ total_auditable_ballots }),
            (true,None) => Audit::OneOnMarginSq(BallotPollingOneOnDilutedMarginSquared{ total_auditable_ballots }),
            (false,Some(confidence)) => Audit::MACRO(BallotComparisonMACRO{total_auditable_ballots,confidence,error_inflation_factor:args.error_inflation_factor.unwrap_or(1.0)}),
            (true,Some(confidence)) => Audit::BRAVO(BallotPollingBRAVO{total_auditable_ballots,confidence}),
        };
        contest.to_raire_problem(audit)?
    };
    let output_file : PathBuf = args.output_json_file.unwrap_or_else(||{
        let mut stem = args.input_raire_file.file_stem().map(|s|PathBuf::from(s)).unwrap_or_else(||PathBuf::from("output"));
        if let Some(contest_ind) = args.contest {
            stem.as_mut_os_string().push(&format!("_{}",contest_ind));
        }
        stem.as_mut_os_string().push(".json");
        stem
    });
    serde_json::to_writer(File::create(&output_file)?,&output)?;
    Ok(())
}
