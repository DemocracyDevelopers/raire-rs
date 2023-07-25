// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! The types of assertions about the election, generally relative standings of various candidates


use crate::audit_type::{AssertionDifficulty, AuditType};
use crate::irv::{BallotPaperCount, CandidateIndex, Votes};
use serde::Deserialize;
use serde::Serialize;

/// Assert that _winner_ beats _loser_ in a winner only audit satisfying the condition
/// that _winner_ gets more first preference votes than _loser_ gets votes when all
/// candidates other than _winner_ and _loser_ are eliminated.
///
/// In other words, there is no way that _loser_ can be eliminated before _winner_.
///
/// This was called WinnerOnly in the original paper.
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
pub struct NotEliminatedBefore {
    pub winner : CandidateIndex,
    pub loser : CandidateIndex,
}

impl NotEliminatedBefore {
    pub fn difficulty<A:AuditType>(&self, votes:&Votes, audit:&A) -> AssertionDifficulty {
        let tally_winner = votes.first_preference_only_tally(self.winner);
        let tallies = votes.restricted_tallies(&vec![self.winner,self.loser]);
        let tally_loser = tallies[1];
        audit.difficulty(tally_winner, tally_loser, tally_winner+tally_loser) // TODO this active paper count seems wrong but produces answers compatible with the paper.
    }

    pub fn find_best_assertion<A:AuditType>(c:CandidateIndex, later_in_pi:&[CandidateIndex], votes:&Votes, audit:&A) -> Option<AssertionAndDifficulty> {
        let mut best_asn = f64::MAX;
        let mut best_assertion : Option<NotEliminatedBefore> = None;
        for alt_c in 0..votes.num_candidates() {
            let alt_c = CandidateIndex(alt_c);
            if alt_c!=c {
                let contest = if later_in_pi.contains(&alt_c) {
                    // consider WO(c,c′): Assertion that c beats c′ ∈ π, where c′ != c appears later in π
                    NotEliminatedBefore {winner:c,loser:alt_c}
                } else {
                    // consider WO(c′′,c): Assertion that c′′ ∈ C\π beats c in a winner-only audit with winner c′′ and loser c
                    NotEliminatedBefore {winner:alt_c,loser:c}
                };
                let asn = contest.difficulty(votes, audit);
                if asn<best_asn {
                    best_asn=asn;
                    best_assertion=Some(contest);
                }
            }
        }
        if let Some(assertion) = best_assertion {
            Some(AssertionAndDifficulty { assertion:Assertion::NEB(assertion), difficulty:best_asn })
        } else {None}
    }

    /// see if the assertion doesn't rule out the given elimination order.
    pub fn ok(&self,elimination_order:&[CandidateIndex]) -> bool {
        // the winner cannot be excluded before the loser.
        check_winner_eliminated_after_loser(elimination_order,self.winner,self.loser)
    }
}

fn check_winner_eliminated_after_loser(elimination_order:&[CandidateIndex],winner:CandidateIndex,loser:CandidateIndex) -> bool {
    if let Some(winner_position) = elimination_order.iter().position(|v|*v==winner) {
        if let Some(loser_position) = elimination_order.iter().position(|v|*v==loser) {
            winner_position>loser_position // compatible with this if the winner was excluded after the loser.
        } else { true } // should never happen if inputs sane
    } else { true } // should never happen if inputs sane
}


/// Assert that _loser_ will be the lowest scoring (and thus candidate to exclude) in an IRV round with the given continuing candidates.
/// If there is more than 1 loser it means that all those losers will be eliminated simultaneously
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct SpecificLoserAmongstContinuing {
    pub continuing: Vec<CandidateIndex>,
    pub losers : Vec<CandidateIndex>,
}

impl SpecificLoserAmongstContinuing {
    pub fn difficulty<A:AuditType>(&self, votes:&Votes, audit:&A) -> AssertionDifficulty {
        let tallies = votes.restricted_tallies(&self.continuing);
        let mut lowest_tally_winner = BallotPaperCount(usize::MAX);
        let mut tally_loser = BallotPaperCount(0);
        for i in 0..self.continuing.len() {
            if self.losers.contains(&self.continuing[i]) { tally_loser+=tallies[i]; }
            else if lowest_tally_winner>tallies[i] { lowest_tally_winner=tallies[i]; }
        }
        audit.difficulty(lowest_tally_winner, tally_loser, tallies.iter().cloned().sum())
    }
}

/// Assert that _winner_ beats _loser_ in an audit when all candidates other that
/// those in _remaining_ have been removed.
///
/// In particular, this means that _winner_ can not be the next candidate eliminated.
///
/// This was called IRV in the original paper.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct NotEliminatedNext {
    pub winner : CandidateIndex,
    pub loser : CandidateIndex,
    /// sorted (ascending) list of continuing candidates.
    pub continuing : Vec<CandidateIndex>,
}

impl NotEliminatedNext {
    pub fn difficulty<A:AuditType>(&self, votes:&Votes, audit:&A) -> AssertionDifficulty {
        let tallies = votes.restricted_tallies(&self.continuing);
        let mut tally_winner = BallotPaperCount(usize::MAX);
        let mut tally_loser = BallotPaperCount(0);
        for i in 0..self.continuing.len() {
            if self.loser==self.continuing[i] { tally_loser=tallies[i]; }
            else if self.winner==self.continuing[i] { tally_winner=tallies[i]; }
        }
        audit.difficulty(tally_winner, tally_loser, tallies.iter().cloned().sum())
    }

    pub fn find_best_difficulty<A:AuditType>(votes:&Votes, audit:&A, continuing:&[CandidateIndex], winner:CandidateIndex) -> Option<AssertionAndDifficulty> {
        let tallies = votes.restricted_tallies(&continuing);
        let mut tally_winner = BallotPaperCount(usize::MAX);
        let mut tally_loser = BallotPaperCount(usize::MAX);
        let mut best_loser  : Option<CandidateIndex> = None;
        //println!("continuing = {:?} tallies={:?}",continuing,tallies);
        for i in 0..continuing.len() {
            if winner==continuing[i] { tally_winner=tallies[i]; }
            else if tallies[i]<=tally_loser { best_loser=Some(continuing[i]);  tally_loser=tallies[i]; }
        }
        if let Some(loser) = best_loser {
            let difficulty = audit.difficulty(tally_winner, tally_loser, tallies.iter().cloned().sum());
            let mut continuing = continuing.to_vec();
            continuing.sort_unstable_by_key(|c|c.0); // important to make it canonical so that equality checks of assertions work, and so is_contining can use a binary search. Also sorted is easier to read.
            let assertion = NotEliminatedNext { winner, loser, continuing };
            Some(AssertionAndDifficulty { assertion:Assertion::NEN(assertion), difficulty })
        } else {None}
    }

    fn is_continuing(&self,c:CandidateIndex) -> bool {
        self.continuing.binary_search_by_key(&c.0,|e|e.0).is_ok()
    }

    /// see if the assertion doesn't rule out the given elimination order.
    pub fn ok(&self,elimination_order:&[CandidateIndex]) -> bool {
        // the order of the people who are left when down to the same length as self.continuing().
        let suffix = &elimination_order[(elimination_order.len()-self.continuing.len())..];
        // check to see the last candidates in the elimination order match the continuing candidates.
        for c in suffix {
            if !self.is_continuing(*c) { return true } // the elimination order is not affected by this rule as the continuing candidates are wrong.
        }
        check_winner_eliminated_after_loser(suffix,self.winner,self.loser) // could pass the whole elimination order, but suffix is fine and faster.
    }
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(tag = "type")]
pub enum Assertion {
    NEB(NotEliminatedBefore),
    NEN(NotEliminatedNext),
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct AssertionAndDifficulty {
    pub assertion : Assertion,
    pub difficulty: f64,
}


impl Assertion {
    pub fn ok(&self,elimination_order:&[CandidateIndex]) -> bool {
        match self {
            Assertion::NEB(wo) => wo.ok(elimination_order),
            Assertion::NEN(irv) => irv.ok(elimination_order),
        }
    }
}

// Code to check what a set of assertions implies.

pub type CandidatePermutation = Vec<CandidateIndex>;

/// Get all num_candidates factorial possible orderings
pub fn all_elimination_orders(num_candidates:u32) -> Vec<CandidatePermutation> {
    if num_candidates==0 { vec![vec![]] }
    else {
        let c = CandidateIndex(num_candidates-1);
        let mut res = vec![];
        for v in all_elimination_orders(num_candidates-1) {
            // put c in every possible place
            for i in 0..=v.len() {
                let mut vv=v.clone();
                vv.insert(i,c);
                res.push(vv);
            }
        }
        res
    }
}



