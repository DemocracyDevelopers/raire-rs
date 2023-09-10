// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! Test the examples given in the new "Guide to RAIRE" document, chapter 6.


use serde_json::json;
use raire::assertions::{NotEliminatedNext, NotEliminatedBefore};
use raire::audit_type::{Audit, BallotComparisonOneOnDilutedMargin};
use raire::irv::{BallotPaperCount, CandidateIndex, Vote, Votes};
use raire::RaireProblem;

const A : CandidateIndex = CandidateIndex(0); // Alice
const B : CandidateIndex = CandidateIndex(1); // Bob
const C : CandidateIndex = CandidateIndex(2); // Chuan
const D : CandidateIndex = CandidateIndex(3); // Diego

/// Get the votes in Example 10 (at the time of writing), used in examples in chapter 6, "Using RAIRE to generate assertions".
fn get_votes() -> Votes {
    let votes = vec![
        Vote{ n: BallotPaperCount(5000), prefs: vec![C,B,A]},
        Vote{ n: BallotPaperCount(1000), prefs: vec![B,C,D]},
        Vote{ n: BallotPaperCount(1500), prefs: vec![D,A]},
        Vote{ n: BallotPaperCount(4000), prefs: vec![A,D]},
        Vote{ n: BallotPaperCount(2000), prefs: vec![D]},
    ];
    Votes::new(votes, 4)
}

/// The audit used in the examples.
const AUDIT : BallotComparisonOneOnDilutedMargin = BallotComparisonOneOnDilutedMargin { total_auditable_ballots : BallotPaperCount(13500) };


#[test]
/// Test the get_votes() function and the methods on the Votes object.
fn test_votes_structure() {
    let votes = get_votes();
    assert_eq!(AUDIT.total_auditable_ballots,votes.total_votes());
    assert_eq!(BallotPaperCount(4000),votes.first_preference_only_tally(CandidateIndex(0)));
    assert_eq!(BallotPaperCount(1000),votes.first_preference_only_tally(CandidateIndex(1)));
    assert_eq!(BallotPaperCount(5000),votes.first_preference_only_tally(CandidateIndex(2)));
    assert_eq!(BallotPaperCount(3500),votes.first_preference_only_tally(CandidateIndex(3)));
    assert_eq!(vec![BallotPaperCount(4000),BallotPaperCount(6000),BallotPaperCount(3500)],votes.restricted_tallies(&vec![CandidateIndex(0),CandidateIndex(2),CandidateIndex(3)]));
    assert_eq!(vec![BallotPaperCount(5500),BallotPaperCount(6000)],votes.restricted_tallies(&vec![CandidateIndex(0),CandidateIndex(2)]));
    let result = votes.run_election();
    assert_eq!(vec![C],result.possible_winners);
    assert_eq!(vec![B,D,A,C],result.elimination_order);
}

#[test]
/// Check NEB assertions in table 6.1 showing that A, B and C cannot be the last candidate standing.
fn test_neb_assertions() {
    let votes = get_votes();
    let test_neb = |winner:CandidateIndex,loser:CandidateIndex| {
        let assertion = NotEliminatedBefore{winner,loser};
        assertion.difficulty(&votes, &AUDIT)
    };
    assert!(test_neb(B,A).is_infinite());
    assert!(test_neb(C,A).is_infinite());
    assert!(test_neb(D,A).is_infinite());
    assert!(test_neb(A,B).is_infinite());
    assert!((test_neb(C,B)-3.375).abs()<0.001);
    assert!(test_neb(D,B).is_infinite());
    assert!(test_neb(A,D).is_infinite());
    assert!(test_neb(B,D).is_infinite());
    assert!(test_neb(C,D).is_infinite());
}

#[test]
/// Check some of the expansions in the tutorial
fn test_expansions() {
    let votes = get_votes();
    let node12 = NotEliminatedNext{
        winner: A,
        loser: D,
        continuing: vec![A,C,D],
    };
    let node12_effort = node12.difficulty(&votes,&AUDIT);
    println!("node12 effort {node12_effort}");
}

#[test]
/// Test RAIRE
fn test_raire() {
    let problem = RaireProblem {
        metadata : json!({
            "candidates" : ["Alice","Bob","Chuan","Diego"]
        }),
        num_candidates : 4,
        votes : get_votes().votes,
        winner : Some(CandidateIndex(2)),
        audit : Audit::Margin(AUDIT)
    };
    println!("{}",serde_json::to_string_pretty(&problem).unwrap());
    let solution = problem.solve();
    println!("{}",serde_json::to_string_pretty(&solution).unwrap());
    // TODO complete.
}

