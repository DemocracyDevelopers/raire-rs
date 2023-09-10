// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


use crate::audit_type::Audit;
use crate::irv::{CandidateIndex, Vote, Votes};
use crate::raire_algorithm::{raire, RaireResult, TrimAlgorithm};
use serde::Deserialize;
use serde::Serialize;

pub mod assertions;
pub mod irv;
pub mod audit_type;
pub mod raire_algorithm;
mod order_assertions;
pub mod tree_showing_what_assertions_pruned_leaves;

#[derive(thiserror::Error, Debug,Serialize,Deserialize,Clone)]
pub enum RaireError {
    #[error("time out - problem too hard")]
    Timeout,
    /// An alternate winner is possible when there are ties. There may be tie resolution legislation
    /// that unambiguously resolves ties, but such a situation where the winner depends upon such
    /// tie resolution is implausible to audit stochastically as a one vote difference would change
    /// the outcome.
    #[error("candidates {0:?} tied as alternate winners")]
    TiedWinners(Vec<CandidateIndex>),
    #[error("the asserted winner was not actually the winner - expecting {0:?}")]
    WrongWinner(Vec<CandidateIndex>),
    #[error("could not rule out the elimination order {0:?}")]
    CouldNotRuleOut(Vec<CandidateIndex>),
    #[error("internal error - ruled out the winner")]
    InternalErrorRuledOutWinner,
    #[error("internal error - did not rule out a loser")]
    InternalErrorDidntRuleOutLoser,
    #[error("internal error - trimming couldn't work")]
    InternalErrorTrimming,
}
/// This file contains an API suitable for a web service.

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct RaireProblem {
    pub metadata : serde_json::Value,
    pub num_candidates : usize,
    pub votes : Vec<Vote>,
    /// Not strictly necessary, only used for consistency checking with the announced winner.
    /// But I recommend it.
    /// We don't want to announce the wrong winner, and then for the audit to prove the winner is the correct person, and no one notice that that was not the person announced.
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub winner : Option<CandidateIndex>,
    pub audit : Audit,
    /// the algorithm used to trim.
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub trim_algorithm : Option<TrimAlgorithm>,
    /// don't bother optimizing below this difficulty level. A value of this > 0 may make the algorithm faster, but may make the results worse, but no worse than this.
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub difficulty_estimate : Option<f64>,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct RaireSolution {
    pub metadata : serde_json::Value,
    pub solution : Result<RaireResult,RaireError>,
}

impl RaireProblem {
    pub fn solve(self) -> RaireSolution {
        let votes = Votes::new(self.votes,self.num_candidates);
        let solution = raire(&votes,self.winner,&self.audit,self.trim_algorithm.unwrap_or(TrimAlgorithm::MinimizeTree));
        RaireSolution{metadata:self.metadata,solution}
    }
}