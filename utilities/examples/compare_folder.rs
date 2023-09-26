// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.

//! Run on all the examples in some folder


use std::fs::File;
use raire::raire_algorithm::TrimAlgorithm;
use raire::RaireProblem;
use utilities::table_of_results::TableOfResults;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env().format_timestamp_millis().filter_level(log::LevelFilter::Trace).init();
    let mut summaries = vec![TableOfResults::default(),TableOfResults::default(),TableOfResults::default(),TableOfResults::default()];
    let folder = "../ConcreteSTVToRaire/NSW Local Government/2012/";
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().ends_with(".json") {
            println!("Processing {:?}",entry.file_name());
            let problem : RaireProblem = serde_json::from_reader(File::open(&entry.path())?)?;
            for (trim,table) in vec![TrimAlgorithm::None,TrimAlgorithm::MinimizeTree,TrimAlgorithm::MinimizeAssertions].into_iter().zip(summaries.iter_mut()) {
                let mut problem = problem.clone();
                problem.trim_algorithm=Some(trim);
                problem.time_limit_seconds=Some(30.0);
                table.push(problem.solve());
            }
        }
    }
    TableOfResults::compare_trims(&summaries);
    Ok(())
}