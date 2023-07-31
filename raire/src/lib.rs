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
use crate::raire_algorithm::{raire, RaireResult};
use serde::Deserialize;
use serde::Serialize;

pub mod assertions;
pub mod irv;
pub mod audit_type;
pub mod raire_algorithm;
mod order_assertions;

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
    CouldNotRuleOut(Vec<CandidateIndex>)
}
/// This file contains an API suitable for a web service.

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct RaireProblem {
    pub metadata : serde_json::Value,
    pub num_candidates : usize,
    pub votes : Vec<Vote>,
    pub winner : CandidateIndex,
    pub audit : Audit,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct RaireSolution {
    pub metadata : serde_json::Value,
    pub solution : Result<RaireResult,RaireError>,
}

impl RaireProblem {
    pub fn solve(self) -> RaireSolution {
        let votes = Votes::new(self.votes,self.num_candidates);
        let solution = raire(&votes,self.winner,&self.audit);
        RaireSolution{metadata:self.metadata,solution}
    }
}