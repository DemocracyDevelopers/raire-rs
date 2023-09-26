// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.



use raire::{RaireSolution};

pub struct SingleResultSummary {
    /// what the contest was
    pub name : String,
    pub solution : RaireSolution,
}

impl SingleResultSummary {
    pub fn new(solution:RaireSolution) -> Self {
        let name = solution.metadata["contest"].to_string();
        SingleResultSummary{name,solution}
    }
}

#[derive(Default)]
pub struct TableOfResults {
    pub results : Vec<SingleResultSummary>,
}

impl TableOfResults {
    pub fn push(&mut self,solution:RaireSolution) {
        self.results.push(SingleResultSummary::new(solution));
    }

    pub fn print(&self) {
        println!("name\tcandidates\tdifficulty\tmargin\tassertions\twinners\tassertions\ttrim");
        for line in &self.results {
            print!("{}\t",line.name);
            match &line.solution.solution {
                Ok(result) => { println!("{}\t{:.3}\t{}\t{}\t{}\t{}\t{}",result.num_candidates,result.difficulty,result.margin,result.assertions.len(),result.time_to_determine_winners.pretty_print(),result.time_to_find_assertions.pretty_print(),result.time_to_trim_assertions.pretty_print())}
                Err(e) => { println!("{}",e)}
            }
        }
    }

    /// Compare trim effects for different trim algorithms
    pub fn compare_trims(algorithms:&[TableOfResults]) {
        for line_index in 0..algorithms[0].results.len() {
            print!("{}\t{}\t{}\t",algorithms[0].results[line_index].name,algorithms[0].results[line_index].solution.solution.as_ref().map(|s|s.num_candidates.to_string()).unwrap_or_default(),algorithms[0].results[line_index].solution.solution.as_ref().map(|s|s.time_to_find_assertions.pretty_print()).unwrap_or_default());
            for a in algorithms {
                print!("{}\t",a.results[line_index].solution.solution.as_ref().map(|s|if s.warning_trim_timed_out {"".to_string()} else {s.assertions.len().to_string()}).unwrap_or_default());
            }
            for a in algorithms {
                print!("{}\t",a.results[line_index].solution.solution.as_ref().map(|s|if s.warning_trim_timed_out {"".to_string()} else {s.time_to_trim_assertions.pretty_print()}).unwrap_or_default());
            }
            println!();
        }
    }
}