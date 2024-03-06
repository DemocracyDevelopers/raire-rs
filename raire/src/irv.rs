// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! The types used in an IRV election
//!
//! Many of these are wrappers around integers, used to prevent e.g. adding a number of votes to a candidate index.
//! Rust allows zero cost abstractions for such wrappers, so there is little reason not to use them.


use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;
use serde::Deserialize;
use serde::Serialize;
use crate::RaireError;
use crate::timeout::TimeOut;

/// A number representing a count of pieces of paper.
#[derive(Copy,Clone,Eq, PartialEq,Serialize,Deserialize,Ord, PartialOrd)]
pub struct BallotPaperCount(pub usize);

impl AddAssign for BallotPaperCount {
    fn add_assign(&mut self, rhs: Self) { self.0+=rhs.0; }
}
impl SubAssign for BallotPaperCount {
    fn sub_assign(&mut self, rhs: Self) { self.0-=rhs.0; }
}

impl Sub for BallotPaperCount {
    type Output = BallotPaperCount;
    fn sub(self, rhs: Self) -> Self::Output { BallotPaperCount(self.0-rhs.0) }
}

impl Add for BallotPaperCount {
    type Output = BallotPaperCount;
    fn add(self, rhs: Self) -> Self::Output { BallotPaperCount(self.0+rhs.0) }
}
// type alias really, don't want long display
impl fmt::Display for BallotPaperCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}
// type alias really, don't want long display
impl fmt::Debug for BallotPaperCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}
/*
impl num::Zero for BallotPaperCount {
    fn zero() -> Self { BallotPaperCount(0) }
    fn is_zero(&self) -> bool { self.0 == 0 }
}*/
impl FromStr for BallotPaperCount {
    type Err = <usize as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BallotPaperCount(s.parse()?))
    }
}
impl Sum for BallotPaperCount {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        BallotPaperCount(usize::sum(iter.map(|b|b.0)))
    }
}

/// a candidate, referred to by position on the ballot paper, 0 being first
#[derive(Clone, Copy, PartialEq, Eq, Hash,Serialize,Deserialize)]
pub struct CandidateIndex(pub u32);
// type alias really, don't want long display
impl fmt::Display for CandidateIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}
// type alias really, don't want long display
impl fmt::Debug for CandidateIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "#{}", self.0) }
}

impl FromStr for CandidateIndex {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CandidateIndex(u32::from_str(s)?))
    }
}

/// a candidate, as part of a subset of candidates, 0 being the first in the subset.
#[derive(Clone, Copy, PartialEq, Eq, Hash,Serialize,Deserialize)]
pub struct SubCandidateIndex(pub u32);
// type alias really, don't want long display
impl fmt::Display for SubCandidateIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}
// type alias really, don't want long display
impl fmt::Debug for SubCandidateIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "#{}", self.0) }
}

impl SubCandidateIndex {
    const INVALID : SubCandidateIndex = SubCandidateIndex(u32::MAX);
}


#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Vote {
    /// The number of voters who voted this way
    pub n : BallotPaperCount,
    /// prefs[0] is the first preferenced candidate.
    pub prefs : Vec<CandidateIndex>,
}

impl Vote {
    /// find the highest preferenced candidate amongst the continuing candidates
    pub fn top_preference(&self,continuing:HashSet<CandidateIndex>) -> Option<CandidateIndex> {
        for c in &self.prefs {
            if continuing.contains(c) { return Some(*c); }
        }
        None
    }
    /// find the highest preferenced candidate amongst the continuing candidates
    pub fn top_sub_preference(&self,continuing:&HashMap<CandidateIndex,SubCandidateIndex>) -> Option<SubCandidateIndex> {
        for c in &self.prefs {
            if let Some(sub) = continuing.get(c) { return Some(*sub) }
        }
        None
    }
    /// find the highest preferenced candidate amongst the continuing candidates
    pub fn top_sub_preference_array(&self,continuing:&[SubCandidateIndex]) -> Option<SubCandidateIndex> {
        for c in &self.prefs {
            if let Some(sub) = continuing.get(c.0 as usize) {
                if *sub!=SubCandidateIndex::INVALID { return Some(*sub) }
            }
        }
        None
    }
}

pub struct Votes {
    pub votes : Vec<Vote>,
    first_preference_votes : Vec<BallotPaperCount>,
}



impl Votes {
    pub fn new(votes:Vec<Vote>,num_candidates:usize) -> Result<Votes,RaireError> {
        let mut first_preference_votes = vec![BallotPaperCount(0);num_candidates];
        for v in &votes {
            if let Some(c) = v.prefs.get(0) {
                if c.0 as usize>=num_candidates { return Err(RaireError::InvalidCandidateNumber); }
                first_preference_votes[c.0 as usize]+=v.n;
            }
        }
        Ok(Votes { votes, first_preference_votes })
    }
    pub fn first_preference_only_tally(&self,candidate:CandidateIndex) -> BallotPaperCount { self.first_preference_votes[candidate.0 as usize] }

    /// Get the tallies for continuing candidates, returning a vector of the same length and order as the continuing structure
    pub fn restricted_tallies(&self,continuing:&[CandidateIndex]) -> Vec<BallotPaperCount> {
        let mut res = vec![BallotPaperCount(0);continuing.len()];
        if continuing.len()>0 {
            //let mut continuing_map : HashMap<CandidateIndex,SubCandidateIndex> = Default::default();
            let mut continuing_map: Vec<SubCandidateIndex> = vec![SubCandidateIndex::INVALID;continuing.iter().map(|v|v.0).max().unwrap() as usize+1];
            for i in 0..continuing.len() {
                // continuing_map.insert(continuing[i],SubCandidateIndex(i as u32));
                continuing_map[continuing[i].0 as usize]=SubCandidateIndex(i as u32);
            }
            for v in &self.votes {
                if let Some(c) = v.top_sub_preference_array(&continuing_map) {
                    res[c.0 as usize]+=v.n;
                }
            }
        }
        res
    }

    pub fn total_votes(&self) -> BallotPaperCount {
        let mut res = BallotPaperCount(0);
        for v in &self.votes {
            res+=v.n;
        }
        res
    }

    pub fn num_candidates(&self) -> u32 { self.first_preference_votes.len() as u32 }

    /// only possible error is RaireError::TimeoutCheckingWinner
    pub fn run_election(&self,timeout:&mut TimeOut) -> Result<IRVResult,RaireError> {
        let mut work = IRVElectionWork{ winner_given_continuing_candidates: Default::default(), elimination_order: vec![] };
        let all_candidates : Vec<CandidateIndex> = (0..self.num_candidates()).into_iter().map(|c|CandidateIndex(c)).collect();
        let possible_winners = work.find_all_possible_winners(all_candidates,&self,timeout)?;
        Ok(IRVResult{ possible_winners, elimination_order: work.elimination_order })
    }

}

/// The result of an IRV election.
pub struct IRVResult {
    /// Possible winners under IRV with no tie resolution. There may be tie resolution rules, but such a close election is not auditable stochastically.
    pub possible_winners : Vec<CandidateIndex>,
    /// A possible elimination order
    pub elimination_order : Vec<CandidateIndex>,
}


struct IRVElectionWork {
    /// Key is a list of continuing candidates, in canonical sorted order.
    /// Value is a list of possible candidates who could win from that point.
    winner_given_continuing_candidates : HashMap<Vec<CandidateIndex>,Vec<CandidateIndex>>,
    /// One order in which candidates are eliminated.
    elimination_order : Vec<CandidateIndex>,
}

impl IRVElectionWork {
    /// Find all possible winners, trying all options with ties, with a set of given continuing votes.
    ///
    /// # Algorithm Complexity
    /// The worst case for all possible elimination orders is n!. An example of this
    /// would be n candidates, each with one vote, just preferencing themself, in which
    /// case all n! elimination orders are plausible. This would make an algorithm
    /// that uses this n! in time complexity (worst case), which is horrible. A simple
    /// simple dynamic programming optimization based on the continuing candidates reduces
    /// this to 2^n, which is a little less horrible, but still occasionally problematic.
    ///
    /// Of course this example is not practical as such an example could not be solved by
    /// RAIRE anyway. However a more likely example of a few candidates with a large number
    /// of votes, and a large number of candidates each with only one vote is more plausible
    /// and causes the same problem, but is easily solvable with RAIRE using NEB assertions.
    ///
    /// There are a variety of solutions to this. Stochastic evaluation - run the election
    /// a million times, with each tie resolved randomly - would do a pretty good job, but
    /// is unreasonably slow in the normal case, is hard to test, and is generally imperfect.
    /// However it is guaranteed not too bad. I hate this idea, but it may be the best option
    /// if this turns out to be a problem in practice.
    ///
    /// What this algorithm does do is to use a special case optimization. Let the candidates
    /// be ranked by tally V_i at some point, so V_i≥V_{i-1}. Compute a cumulative sum
    /// S_i = ∑_{j⩽i} V_j. Then if V_i>S_{i-1} we can say that no matter how the preferences
    /// of the votes going to candidates up to and including i-1 go, no candidate i or above
    /// will be excluded before all candidates up to and including i-1 have been excluded. Thus
    /// one can exclude all candidates up to i-1 at this point without worrying about their
    /// order. This *bulk elimination* doesn't solve all cases, but does solve the most likely
    /// problematic case of a few candidates with lots of votes and a large number of candidates
    /// with a tiny number of votes.
    ///
    /// Also note that if bulk elimination is used, the example elimination order may not be exact.
    /// This is simply remedied (at gratuitous but not exponential computational cost)
    /// by just excluding one candidate from the bulk elimination - one of the ones with
    /// lowest tally. For computational reasons, bulk elimination is only tried in the case of ties.
    ///
    fn find_all_possible_winners(&mut self,continuing:Vec<CandidateIndex>,votes:&Votes,timeout:&mut TimeOut) -> Result<Vec<CandidateIndex>,RaireError> {
        if timeout.quick_check_timeout() { return Err(RaireError::TimeoutCheckingWinner); }
        Ok(if continuing.len()==1 {
            if self.elimination_order.len()+continuing.len()==votes.num_candidates() as usize {
                // There may be multiple elimination orders. The check above checks that we are in the path of the first depth first traversal of the tree of elimination orders.
                self.elimination_order.push(continuing[0]);
            }
            continuing
        } else if let Some(already_computed) = self.winner_given_continuing_candidates.get(&continuing) {
            already_computed.clone()
        } else {
            let tallies = votes.restricted_tallies(&continuing);
            let min_tally = *tallies.iter().min().unwrap();
            let mut winners = HashSet::new();
            let mut already_tried_one_option = false;
            let mut already_tried_bulk_elimination = false;
            for i in 0..continuing.len() {
                if min_tally==tallies[i] { // this is a plausible candidate to exclude. There may be a tie in which case there are multiple options. Try them all.
                    if already_tried_one_option && !already_tried_bulk_elimination {
                        // check to see if bulk elimination is an option. If so, don't bother trying any more candidates.
                        if Self::find_bulk_elimination(&continuing,&tallies).is_some() { break; }
                        already_tried_bulk_elimination=true;
                    }
                    if self.elimination_order.len()+continuing.len()==votes.num_candidates() as usize {
                        // There may be multiple elimination orders. The check above checks that we are in the path of the first depth first traversal of the tree of elimination orders.
                        self.elimination_order.push(continuing[i]);
                    }
                    let mut new_continuing = continuing[0..i].to_vec();
                    new_continuing.extend_from_slice(&continuing[i+1..]);
                    let res = self.find_all_possible_winners(new_continuing,votes,timeout)?;
                    for c in res { winners.insert(c); }
                    already_tried_one_option=true
                }
            }
            let winners : Vec<CandidateIndex> = winners.into_iter().collect();
            self.winner_given_continuing_candidates.insert(continuing,winners.clone());
            winners
        })
    }

    /// Compute a set of at least 2 candidates to eliminate, if possible, using the
    /// bulk elimination algorithm described in the docs for find_all_possible_winners.
    ///
    /// If it finds such a set, they are returned, sorted in order from smallest tally to largest tally.
    ///
    /// Note: In practice, a boolean return value would work, in which case we only need to sort tallies, which would be faster.
    fn find_bulk_elimination(continuing:&[CandidateIndex],tallies:&[BallotPaperCount]) -> Option<Vec<CandidateIndex>> {
        let mut merged : Vec<(CandidateIndex,BallotPaperCount)> = continuing.iter().copied().zip(tallies.iter().copied()).collect();
        merged.sort_by_key(|(_,t)|t.0);
        let mut cumulative_sum : BallotPaperCount = BallotPaperCount(0);
        for i in 0..merged.len() {
            if i>1 && merged[i].1>cumulative_sum { // can do bulk exclusion of candidates 0 inclusive to i exclusive
                let bulk_elimination : Vec<CandidateIndex> = merged[0..i].iter().map(|(c,_)|c).copied().collect();
                return Some(bulk_elimination)
            }
            cumulative_sum+=merged[i].1;
        }
        None
    }
}
