// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.

//! Parse the contests in the USIRV folder of https://github.com/michelleblom/audit-irv-cp/tree/raire-branch


use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::anyhow;
use serde_json::json;
use raire::audit_type::Audit;
use raire::irv::{BallotPaperCount, CandidateIndex, Vote, Votes};
use raire::RaireProblem;

pub struct Contest {
    pub num_candidates : usize,
    pub id : String,
    pub candidate_names : Vec<String>,
    pub candidate_name_to_index : HashMap<String,CandidateIndex>,
    pub votes : HashMap<Vec<CandidateIndex>,usize>,
}


impl Contest {
    pub fn parse<P: AsRef<Path>>(path:P) -> anyhow::Result<Vec<Contest>> {
        let mut lines = BufReader::new(File::open(path)?).lines();
        // first line is number of contests
        let num_contests : usize = lines.next().ok_or_else(||anyhow!("No number of contests on first line"))??.parse()?;
        println!("File contains {num_contests} contests.");
        let mut res = vec![];
        for i in 0..num_contests {
            let line = lines.next().ok_or_else(||anyhow!("Missing contest {}",i+1))??;
            let fields = line.split(',').collect::<Vec<_>>();
            // first field is typically "Contest" then and id then number of candidates, then candidate names
            if fields.len()<3 { return Err(anyhow!("Contest {} doesn't have enough fields",i+1)); }
            let id = fields[1].to_string();
            let num_candidates : usize = fields[2].parse()?;
            let candidate_names : Vec<String> = if fields.len()>=3+num_candidates {
                fields[3..(3+num_candidates)].iter().map(|s|s.to_string()).collect()
            } else { return Err(anyhow!("Candidate ids missing")); };
            let candidate_name_to_index : HashMap<String,CandidateIndex> = candidate_names.iter().enumerate().map(|(n,name)|(name.clone(),CandidateIndex(n as u32))).collect();
            res.push(Contest{num_candidates,id,candidate_names,candidate_name_to_index,votes:Default::default()});
        }
        // rest of lines are contest,ballot_id,candidates (starting from 1)
        for line in lines {
            let line = line?;
            let mut fields = line.split(',');
            if let Some(contest_id) = fields.next() {
                if let Some(contest) = res.iter_mut().find(|c|c.id.as_str()==contest_id) {
                    if let Some(_ballot_id) = fields.next() {
                        let remaining = fields.collect::<Vec<_>>();
                        let candidates : Vec<CandidateIndex> = if remaining.len()==1 && remaining[0].is_empty() { vec![] } else {
                            remaining.iter().map(|&s|*contest.candidate_name_to_index.get(s).expect("Expected integer candidate id")).collect()
                        };
                        *contest.votes.entry(candidates).or_insert(0)+=1;
                    }
                }
            }
        }
        Ok(res)
    }
    pub fn to_raire_problem(&self,audit : Audit) -> anyhow::Result<RaireProblem> {
        let mut votes : Vec<Vote> = vec![];
        for (prefs,n) in &self.votes {
            votes.push(Vote{ n: BallotPaperCount(*n), prefs:prefs.clone() });
        }
        let votes = Votes::new(votes,self.num_candidates);
        let winners = votes.run_election();
        if winners.possible_winners.len()!=1 { return Err(anyhow!("RAIRE only works if there is one possible winner."))}
        let winner = winners.possible_winners[0];
        let metadata = json!({"candidates":self.candidate_names,"contest":self.id.clone()});
        Ok(RaireProblem{
            metadata,
            num_candidates: self.num_candidates,
            votes: votes.votes,
            winner,
            audit,
        })
    }
}