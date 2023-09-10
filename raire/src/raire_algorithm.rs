// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.

//! The main RAIRE algorithm.



use std::cmp::Ordering;
use std::collections::BinaryHeap;
use crate::assertions::{all_elimination_orders, Assertion, AssertionAndDifficulty, NotEliminatedNext, NotEliminatedBefore, EliminationOrder, EliminationOrderSuffix, EffectOfAssertionOnEliminationOrderSuffix, NotEliminatedBeforeCache};
use crate::audit_type::{AssertionDifficulty, AuditType};
use crate::irv::{CandidateIndex, Votes};
use serde::Deserialize;
use serde::Serialize;
use crate::RaireError;
use crate::tree_showing_what_assertions_pruned_leaves::{HowFarToContinueSearchTreeWhenPruningAssertionFound, TreeNodeShowingWhatAssertionsPrunedIt};

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct RaireResult {
    pub assertions : Vec<AssertionAndDifficulty>,
    pub difficulty: AssertionDifficulty,
    pub winner : CandidateIndex,
    pub num_candidates : u32,
}

impl RaireResult {
    pub fn possible_elimination_orders_allowed_by_assertions(&self,num_candidates:u32) -> Vec<EliminationOrder> {
        let mut elimination_orders = all_elimination_orders(num_candidates);
        for a in &self.assertions {
            elimination_orders.retain(|order|a.assertion.ok_elimination_order_suffix(order)==EffectOfAssertionOnEliminationOrderSuffix::Ok);
        }
        elimination_orders
    }

    pub fn possible_elimination_order_suffixes_allowed_by_assertions(&self,num_candidates:u32) -> Vec<EliminationOrder> {
        let mut elimination_orders : Vec<EliminationOrderSuffix> = vec![vec![]]; // start off with the minimal set.
        for a in &self.assertions {
            let mut next = vec![];
            for v in elimination_orders.drain(..) {
                next.append(&mut a.assertion.allowed_suffixes(v,num_candidates));
            }
            elimination_orders = next;
        }
        elimination_orders
    }

    pub fn verify_result_does_prove_winner(&self) -> Result<(),RaireError> {
        let all_assertions : Vec<Assertion> = self.assertions.iter().map(|ad|ad.assertion.clone()).collect();
        let all_assertion_indices : Vec<usize> = (0..all_assertions.len()).collect();
        for candidate in 0..self.num_candidates {
            let candidate = CandidateIndex(candidate);
            let tree = TreeNodeShowingWhatAssertionsPrunedIt::new(&[],candidate,&all_assertion_indices,&all_assertions,self.num_candidates,HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately);
            if tree.valid!= (candidate==self.winner) { return Err(if candidate==self.winner { RaireError::InternalErrorRuledOutWinner} else { RaireError::InternalErrorDidntRuleOutLoser })}
        }
        Ok(())
    }
}

#[derive(Debug)]
/// An entry in the priority queue.
struct SequenceAndEffort {
    /// a permutation that needs to be ruled out.
    pi : EliminationOrderSuffix,
    best_assertion_for_ancestor : AssertionAndDifficulty,
    /// the best ancestor for pi will be a subset of pi, in particular the last best_ancestor_length elements of pi.
    best_ancestor_length : usize,
    /// if not null, then a dive has already been done on the specified candidate.
    dive_done : Option<CandidateIndex>,
}

impl SequenceAndEffort {
    /// higher means more effort needed
    pub fn difficulty(&self) -> f64 { self.best_assertion_for_ancestor.difficulty }

    /// get the best ancestor of pi, which is a subset of pi.
    pub fn best_ancestor(&self) -> &[CandidateIndex] {
        &self.pi[(self.pi.len()-self.best_ancestor_length)..]
    }

    pub fn extend_by_candidate<A:AuditType>(&self,c:CandidateIndex,votes:&Votes,audit:&A,neb_cache:&NotEliminatedBeforeCache)-> Self {
        let mut pi_prime = vec![c];
        pi_prime.extend_from_slice(&self.pi); // π ′ ← [c] ++π
        let a : AssertionAndDifficulty = find_best_audit(&pi_prime, votes, audit,neb_cache); // a in the original paper
        let (best_ancestor_length,best_assertion_for_ancestor) = if a.difficulty < self.difficulty() { (pi_prime.len(), a.clone()) } else { (self.best_ancestor_length, self.best_assertion_for_ancestor.clone()) };
        SequenceAndEffort { pi:pi_prime, best_ancestor_length, best_assertion_for_ancestor, dive_done: None }
    }

    /// Called when the only use for this is to take the assertion and add it to the list of assertions.
    /// This checks that it is not already there and removes elements from the frontier that obviously match it.
    pub fn just_take_assertion(self,assertions:&mut Vec<AssertionAndDifficulty>,frontier:&mut BinaryHeap<SequenceAndEffort>) {
        if assertions.iter().any(|a|a.assertion==self.best_assertion_for_ancestor.assertion) {
            //println!("Didn't add assertion as it was already there");
        } else {
            //println!("Just including it");
            let best_ancestor_pi = self.best_ancestor();
            // 15 F ← F \ {π ′ ∈ F | ba[π] is a suffix of π ′ }
            frontier.retain(|s|!s.pi.ends_with(best_ancestor_pi));
            // 14 A ← A ∪ {asr[ba[π]]}
            assertions.push(self.best_assertion_for_ancestor);
            // step 14 is done after 15 for lifetime reasons.
        }
    }

    /// Called when a sequence has gone as far as it can - i.e. all candidates are in the exclusion order list.
    pub fn contains_all_candidates(self,assertions:&mut Vec<AssertionAndDifficulty>,frontier:&mut BinaryHeap<SequenceAndEffort>,bound:&mut f64) -> Result<(),RaireError> {
        if self.difficulty().is_infinite() { // 23 if (ASN (asr[ba[π ′ ]]) = ∞):
            //println!("Couldn't deal with {:?}",new_sequence.pi);
            Err(RaireError::CouldNotRuleOut(self.pi)) // 24 terminate algorithm, full recount necessary
        } else {
            if *bound<self.difficulty() {
                *bound=self.difficulty(); // 27 LB ← max(LB, ASN (asr[ba[π′]]))
                log::trace!("Found bound {} on elimination sequence {:?}",*bound,self.pi)
            }
            self.just_take_assertion(assertions,frontier); // Steps 26 and 28 are same as 14 and 15.
            Ok(())
        }
    }
}

// impls for SequenceAndEffort are to support ordering for the priority queue.

impl PartialOrd<Self> for SequenceAndEffort {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.difficulty().partial_cmp(&other.difficulty())
    }
}

impl Eq for SequenceAndEffort {}

impl PartialEq<Self> for SequenceAndEffort {
    fn eq(&self, other: &Self) -> bool {
        self.pi==other.pi && self.difficulty()==other.difficulty()
    }
}

impl Ord for SequenceAndEffort {
    fn cmp(&self, other: &Self) -> Ordering {
        self.difficulty().partial_cmp(&other.difficulty()).unwrap_or(Ordering::Equal) // should always unwrap fine as NaN etc not allowed.
    }
}

fn find_best_audit<A:AuditType>(pi:&[CandidateIndex],votes:&Votes,audit:&A,neb_cache:&NotEliminatedBeforeCache) -> AssertionAndDifficulty {
    let c = pi[0];
    let mut res : AssertionAndDifficulty = AssertionAndDifficulty { assertion: Assertion::NEB(NotEliminatedBefore { winner: c, loser: c }), difficulty: f64::INFINITY }; // dummy infinitely bad assertion
    // consider WO contests
    if let Some(assertion) = NotEliminatedBefore::find_best_assertion_using_cache(c, &pi[1..],votes, neb_cache) {
        if assertion.difficulty < res.difficulty { res=assertion; }
    }
    // consider IRV(c,c′,{c′′ | c′′ ∈ π}): Assertion that c beats some c′ != c ∈ π
    if let Some(assertion) = NotEliminatedNext::find_best_difficulty(votes, audit, pi, c) {
        //println!("{:?}",assertion);
        if assertion.difficulty < res.difficulty { res=assertion; }
    }
    //println!("FindBestAudit({:?})={:?}",pi,res);
    res
}

/// If true, use Michelle's diving search order optimization.
/// Testing shows that it is almost always a moderate improvement in speed.
const USE_DIVING : bool = true;

pub fn raire<A:AuditType>(votes:&Votes,winner:Option<CandidateIndex>,audit:&A,trim_algorithm:TrimAlgorithm) -> Result<RaireResult,RaireError> {
    log::debug!("Starting raire with {} candidates and {} distinct votes",votes.num_candidates(),votes.votes.len());
    let irv_result = votes.run_election();
    if let Some(winner) = winner {
        if !irv_result.possible_winners.contains(&winner) { return Err(RaireError::WrongWinner(irv_result.possible_winners))}
    }
    if irv_result.possible_winners.len()!=1 { return Err(RaireError::TiedWinners(irv_result.possible_winners))}
    let winner : CandidateIndex = irv_result.possible_winners[0]; // replace option by actual value.
    log::debug!("IRV winner {} elimination order {:?}",winner,irv_result.elimination_order);
    let neb_cache = NotEliminatedBeforeCache::new(votes,audit);
    log::trace!("Created NEB cache");
    //println!("Calling raire with {} votes {} candidates winner {}",votes.total_votes(),votes.num_candidates(),winner);
    let mut assertions : Vec<AssertionAndDifficulty> = vec![]; // A in the original paper
    let mut bound : AssertionDifficulty = 0.0; // LB in the original paper
    let mut frontier = BinaryHeap::new(); // F in the original paper
    let mut last_difficulty:f64 = f64::INFINITY;
    // Populate F with single-candidate sequences
    for c in 0..votes.num_candidates() {
        let c = CandidateIndex(c);
        if c!=winner { // 4 for each(c ∈ C \ {c w }):
            let pi = vec![c];
            //  asr[π] ← a ⊲ Record best assertion for π
            let best_assertion_for_pi = find_best_audit(&pi,votes,audit,&neb_cache);  // a in the original paper
            //  ba[π] ← π ⊲ Record best ancestor sequence for π
            let best_ancestor_length = pi.len();
            frontier.push(SequenceAndEffort{pi,best_ancestor_length,best_assertion_for_ancestor:best_assertion_for_pi, dive_done: None }); // difficulty comes from asr[π].
        }
    }
    // Repeatedly expand the sequence with largest ASN in F
    while let Some(mut sequence_being_considered) = frontier.pop() { // 10-12
        if sequence_being_considered.difficulty()!=last_difficulty {
            last_difficulty=sequence_being_considered.difficulty();
            log::trace!("Difficulty reduced to {}{}",last_difficulty,if last_difficulty<=bound {" OK"} else {""});
        }
        //println!("Considering {:?}",sequence_being_considered);
        if sequence_being_considered.difficulty()<=bound { // may as well just include.
            sequence_being_considered.just_take_assertion(&mut assertions,&mut frontier);
        } else {
            if USE_DIVING && !sequence_being_considered.dive_done.is_some() {
                let mut last : Option<SequenceAndEffort> = None;
                assert_eq!(irv_result.elimination_order.len(),votes.num_candidates() as usize);
                for &c in irv_result.elimination_order.iter().rev() {
                    if !sequence_being_considered.pi.contains(&c) {
                        let new_sequence = match last.take() { // don't repeat work! Mark that this path has already been dealt with.
                            Some(mut l) => {
                                l.dive_done=Some(c);
                                let new_sequence = l.extend_by_candidate(c,votes,audit,&neb_cache);
                                frontier.push(l);
                                new_sequence
                            }
                            None => {
                                sequence_being_considered.dive_done=Some(c);
                                sequence_being_considered.extend_by_candidate(c,votes,audit,&neb_cache)
                            },
                        };
                        if new_sequence.difficulty()<=bound {
                            new_sequence.just_take_assertion(&mut assertions,&mut frontier);
                            break;
                        } else {
                            last = Some(new_sequence);
                        }
                    }
                }
                if let Some(last) = last {
                    assert_eq!(last.pi.len(),votes.num_candidates() as usize);
                    last.contains_all_candidates(&mut assertions,&mut frontier,&mut bound)?;
                }
            }
            for c in 0..votes.num_candidates() { // for each(c ∈ C \ π):
                let c = CandidateIndex(c);
                if !(sequence_being_considered.pi.contains(&c)||sequence_being_considered.dive_done==Some(c)) {
                    let new_sequence = sequence_being_considered.extend_by_candidate(c,votes,audit,&neb_cache);
                    if new_sequence.pi.len()==votes.num_candidates() as usize { // 22 if (|π′| = |C|):
                        new_sequence.contains_all_candidates(&mut assertions,&mut frontier,&mut bound)?;
                    } else {
                        frontier.push(new_sequence) // 31 F ← F ∪ {π ′ }
                    }
                }
            }
        }
        //println!("frontier now includes {} elements",frontier.len())
    }
    log::debug!("Finished generating {} assertions difficulty {}, now need to trim.",assertions.len(),bound);
    match trim_algorithm {
        TrimAlgorithm::None => {}
        TrimAlgorithm::Slow => {
            crate::order_assertions::order_assertions_and_remove_unnecessary(&mut assertions,winner,votes.num_candidates());
        }
        TrimAlgorithm::MinimizeTree => {
            crate::tree_showing_what_assertions_pruned_leaves::order_assertions_and_remove_unnecessary(&mut assertions,winner,votes.num_candidates(),HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately)?;
        }
        TrimAlgorithm::MinimizeAssertions => {
            crate::tree_showing_what_assertions_pruned_leaves::order_assertions_and_remove_unnecessary(&mut assertions,winner,votes.num_candidates(),HowFarToContinueSearchTreeWhenPruningAssertionFound::Forever)?;
        }
    }
    log::debug!("Trimmed assertions down to {}.",assertions.len());
    Ok(RaireResult{assertions, difficulty: bound , winner,num_candidates:votes.num_candidates() })
}

#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
/// After the RAIRE algorithm has generated the assertions, it is possible that there are redundant assertions.
///
/// This could happen as the algorithm found some assertion to trim one path, and then later some other
/// assertion is added to trim some other path, but it turns out that it also trims the path trimmed earlier
/// by some other assertion.
///
/// There are a variety of algorithms for removing redundant assertions. It depends what you want to minimize.
///
/// Example: In the very simple case given in the "Guide to RAIRE", there
/// are the assertions (and difficulties):
/// ```text
/// A1: 3     NEN: Alice > Diego if only {Alice,Diego} remain
/// A2: 27    NEN: Chuan > Alice if only {Alice,Chuan} remain
/// A3: 27    NEN: Alice > Diego if only {Alice,Chuan,Diego} remain
/// A4: 5.4   NEN: Chuan > Diego if only {Alice,Chuan,Diego} remain
/// A5: 4.5   NEN: Alice > Bob if only {Alice,Bob,Chuan,Diego} remain
/// A6: 3.375 Chuan NEB Bob
/// ```
///
/// The elimination order `[...Alice,Diego]` is eliminated by `A1`.
///
/// However, `[...,Bob,Alice,Diego]` is eliminated by `A6`, and
/// `[Chuan,Alice,Diego]` is eliminated by `A4`.
///
/// So `A1` is technically unnecessary to prove who is elected, and `A6` and `A4`
/// are both needed elsewhere. But `A1` is necessary if you want to minimize
/// the elimination tree size.
///
/// It is not clear what we want to minimize. A larger number of assertions
/// for a smaller tree is easier for a human to verify (probably), but has
/// a slightly higher chance of requiring an escalation.
///
/// This gives you options. `MinimizeTree` (and `None`) will leave in `A1`, but
/// `MinimizeAssertions` will remove `A1`.
pub enum TrimAlgorithm {
    /// Don't do any trimming
    None,
    /// You probably don't want to do this. It is the original algorithm I came up with which is gratuitously slow and not optimal. Left temporarily for historical reasons, will be deleted soon.
    Slow,
    /// Expand the tree until an assertion rules the path out, removing redundant assertions with a simple heuristic. Minimizes size of tree for human to verify, but may have unnecessary assertions.
    MinimizeTree,
    /// Expand the tree until all all assertions are resolved, and remove redundant assertions with a simple heuristic. Minimizes the number of assertions, but may increase the size of the tree to verify.
    MinimizeAssertions,
}