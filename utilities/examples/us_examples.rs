// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.

//! Run on all the examples in Michelle's repository https://github.com/michelleblom/audit-irv-cp/tree/raire-branch. Requires that that be cloned at the same level as raire-rs


use std::time::SystemTime;
use raire::audit_type::{Audit, BallotComparisonOneOnDilutedMargin};
use raire::irv::BallotPaperCount;
use raire::raire_algorithm::TrimAlgorithm;
use utilities::parse_michelle_format::Contest;
use utilities::table_of_results::TableOfResults;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env().format_timestamp_millis().filter_level(log::LevelFilter::Trace).init();
    let mut summary = TableOfResults::default();
    let mut summaries = vec![TableOfResults::default(),TableOfResults::default(),TableOfResults::default(),TableOfResults::default()];
    let folder = "../audit-irv-cp-raire-branch/USIRV/";
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().ends_with(".raire") {
            let contests = Contest::parse(&entry.path())?;
            for contest_index in 0..contests.len() {
                let contest = &contests[contest_index];
                let num_ballots : usize = contest.votes.values().sum();
                let mut problem = contests[contest_index].to_raire_problem(Audit::OneOnMargin(BallotComparisonOneOnDilutedMargin{total_auditable_ballots:BallotPaperCount(num_ballots)}))?;
                for (trim,table) in vec![TrimAlgorithm::None,TrimAlgorithm::MinimizeTree,TrimAlgorithm::MinimizeAssertions].into_iter().zip(summaries.iter_mut()) {
                    let mut problem = problem.clone();
                    problem.trim_algorithm=Some(trim);
                    table.push(problem.solve());
                }
                problem.trim_algorithm=Some(TrimAlgorithm::MinimizeAssertions2);
                println!("{} contest {} with {} candidates {} ballots of which {} are distinct",entry.file_name().to_string_lossy(),contest_index+1,problem.num_candidates,num_ballots,problem.votes.len());
                let time_start = SystemTime::now();
                let solution = problem.solve();
                let time_taken = SystemTime::now().duration_since(time_start)?;
                match & solution.solution {
                    Ok(result) => {
                        println!("Solved {} contest {} in {:?} difficulty {} with {} assertions",entry.file_name().to_string_lossy(),contest_index+1,time_taken,result.difficulty,result.assertions.len());
                        result.verify_result_does_prove_winner()?;
                    }
                    Err(error) => {
                        println!("Could not solve in {:?} reason {}",time_taken,error);
                    }
                }
                summary.push(solution);
            }
        }
    }
    summary.print();
    println!();
    TableOfResults::compare_trims(&summaries);
    Ok(())
}