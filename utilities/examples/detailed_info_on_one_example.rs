// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.

//! Get detailed information on one particular file.


use std::fs::File;
use raire::raire_algorithm::{TrimAlgorithm};
use raire::{RaireProblem};

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env().format_timestamp_millis().filter_level(log::LevelFilter::Trace).init();
    let file = "../ConcreteSTVToRaire/NSW Local Government/2012/Canterbury City Council - East Ward.json";
    let mut problem : RaireProblem = serde_json::from_reader(File::open(file)?)?;
    problem.trim_algorithm=Some(TrimAlgorithm::MinimizeTree);
    problem.time_limit_seconds=Some(30.0);
    let solution = problem.solve();
    match &solution.solution {
        Ok(result) => {
            println!("Succeeded with result difficulty {} margin {} with {} assertions",result.difficulty,result.margin,result.assertions.len());
            println!("Winner determination : {}s {} operations",result.time_to_determine_winners.seconds,result.time_to_determine_winners.work);
            println!("Assertion generation : {}s {} operations",result.time_to_find_assertions.seconds,result.time_to_find_assertions.work);
            println!("Trim assertions : {}s {} operations",result.time_to_trim_assertions.seconds,result.time_to_trim_assertions.work);
            if result.warning_trim_timed_out { println!("** WARNING ** trim timed out"); }
        }
        Err(e) => {println!("Error : {}",e);}
    }
    Ok(())
}