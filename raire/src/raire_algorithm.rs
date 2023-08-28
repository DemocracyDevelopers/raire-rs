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
use crate::assertions::{all_elimination_orders, Assertion, AssertionAndDifficulty, NotEliminatedNext, NotEliminatedBefore, EliminationOrder, EliminationOrderSuffix, EffectOfAssertionOnEliminationOrderSuffix};
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
}

impl SequenceAndEffort {
    /// higher means more effort needed
    pub fn difficulty(&self) -> f64 { self.best_assertion_for_ancestor.difficulty }

    /// get the best ancestor of pi, which is a subset of pi.
    pub fn best_ancestor(&self) -> &[CandidateIndex] {
        &self.pi[(self.pi.len()-self.best_ancestor_length)..]
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

fn find_best_audit<A:AuditType>(pi:&[CandidateIndex],votes:&Votes,audit:&A) -> AssertionAndDifficulty {
    let c = pi[0];
    let mut res : AssertionAndDifficulty = AssertionAndDifficulty { assertion: Assertion::NEB(NotEliminatedBefore { winner: c, loser: c }), difficulty: f64::INFINITY }; // dummy infinitely bad assertion
    // consider WO contests
    if let Some(assertion) = NotEliminatedBefore::find_best_assertion(c, &pi[1..], votes, audit) {
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

pub fn raire<A:AuditType>(votes:&Votes,winner:CandidateIndex,audit:&A,trim_algorithm:TrimAlgorithm) -> Result<RaireResult,RaireError> {
    log::debug!("Starting raire with {} candidates and {} distinct votes",votes.num_candidates(),votes.votes.len());
    let irv_result = votes.run_election();
    if !irv_result.possible_winners.contains(&winner) { return Err(RaireError::WrongWinner(irv_result.possible_winners))}
    if irv_result.possible_winners.len()!=1 { return Err(RaireError::TiedWinners(irv_result.possible_winners))}
    log::debug!("IRV winner {} elimination order {:?}",winner,irv_result.elimination_order);
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
            let best_assertion_for_pi = find_best_audit(&pi,votes,audit);  // a in the original paper
            //  ba[π] ← π ⊲ Record best ancestor sequence for π
            let best_ancestor_length = pi.len();
            frontier.push(SequenceAndEffort{pi,best_ancestor_length,best_assertion_for_ancestor:best_assertion_for_pi}); // difficulty comes from asr[π].
        }
    }
    // Repeatedly expand the sequence with largest ASN in F
    while let Some(sequence_being_considered) = frontier.pop() { // 10-12
        if sequence_being_considered.difficulty()!=last_difficulty {
            last_difficulty=sequence_being_considered.difficulty();
            log::trace!("Difficulty reduced to {}{}",last_difficulty,if last_difficulty<=bound {" OK"} else {""});
        }
        //println!("Considering {:?}",sequence_being_considered);
        let pi = &sequence_being_considered.pi;
        if sequence_being_considered.difficulty()<=bound { // may as well just include.
            if assertions.iter().any(|a|a.assertion==sequence_being_considered.best_assertion_for_ancestor.assertion) {
                //println!("Didn't add assertion as it was already there");
            } else {
                //println!("Just including it");
                let best_ancestor_pi = sequence_being_considered.best_ancestor();
                // 15 F ← F \ {π ′ ∈ F | ba[π] is a suffix of π ′ }
                frontier.retain(|s|!s.pi.ends_with(best_ancestor_pi));
                // 14 A ← A ∪ {asr[ba[π]]}
                assertions.push(sequence_being_considered.best_assertion_for_ancestor);
                // step 14 is done after 15 for lifetime reasons.
            }
        } else {
            // TODO implement diving.
            for c in 0..votes.num_candidates() { // for each(c ∈ C \ π):
                let c = CandidateIndex(c);
                if !sequence_being_considered.pi.contains(&c) {
                    let mut pi_prime = vec![c];
                    pi_prime.extend_from_slice(pi); // π ′ ← [c] ++π
                    let a : AssertionAndDifficulty = find_best_audit(&pi_prime, votes, audit); // a in the original paper
                    let (best_ancestor_length,best_assertion_for_ancestor) = if a.difficulty < sequence_being_considered.difficulty() { (pi_prime.len(), a.clone()) } else { (sequence_being_considered.best_ancestor_length, sequence_being_considered.best_assertion_for_ancestor.clone()) };
                    let new_sequence = SequenceAndEffort { pi:pi_prime, best_ancestor_length, best_assertion_for_ancestor };
                    if new_sequence.pi.len()==votes.num_candidates() as usize { // 22 if (|π′| = |C|):
                        if new_sequence.difficulty().is_infinite() { // 23 if (ASN (asr[ba[π ′ ]]) = ∞):
                            //println!("Couldn't deal with {:?}",new_sequence.pi);
                            return Err(RaireError::CouldNotRuleOut(new_sequence.pi)); // 24 terminate algorithm, full recount necessary
                        } else {
                            if assertions.iter().any(|a|a.assertion==new_sequence.best_assertion_for_ancestor.assertion) {
                                //println!("Didn't add assertion as it was already there");
                            } else {
                                //println!("Adding {:?} as no choice",new_sequence);
                                if bound<new_sequence.difficulty() {
                                    bound=new_sequence.difficulty(); // 27 LB ← max(LB, ASN (asr[ba[π′]]))
                                }
                                if bound==new_sequence.difficulty() {
                                    log::trace!("Found bound {} on elimination sequence {:?}",bound,new_sequence.pi)
                                }
                                let suffix = new_sequence.best_ancestor();
                                // 28 F ← F \ {π ′ ∈ F | ba[π] is a suffix of π′ }
                                frontier.retain(|s|!s.pi.ends_with(suffix));
                                assertions.push(new_sequence.best_assertion_for_ancestor); // 26 A ← A ∪ {asr[ba[π′]]}
                                // 26 is done after 28 for lifetime reasons.
                            }
                        }
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