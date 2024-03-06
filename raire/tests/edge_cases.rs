// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! Test some edge cases (and check that some inputs correctly produce errors)




use serde_json::json;
use raire::audit_type::{Audit, BallotComparisonOneOnDilutedMargin};
use raire::irv::{BallotPaperCount, CandidateIndex, Vote};
use raire::raire_algorithm::TrimAlgorithm;
use raire::{RaireError, RaireProblem};

#[test]
/// Test 0 candidates... should produce RaireError::InvalidCandidateNumber
fn test_zero_candidates() {
    let problem = RaireProblem {
        metadata : json!({
            "candidates" : ["Alice","Bob","Chuan","Diego"]
        }),
        num_candidates : 0,
        votes : vec![],
        winner : None,
        audit : Audit::OneOnMargin(BallotComparisonOneOnDilutedMargin { total_auditable_ballots : BallotPaperCount(0) }),
        trim_algorithm: Some(TrimAlgorithm::MinimizeAssertions),
        difficulty_estimate: None,
        time_limit_seconds: None,
    };
    let solution = problem.solve();
    match solution.solution {
        Err(RaireError::InvalidNumberOfCandidates) => {}
        _ => panic!("Expecting invalid candidate number, got {}",serde_json::to_string_pretty(&solution.solution).unwrap())
    }
}


#[test]
/// Test 1 candidate... should produce a valid winner even with no votes
fn test_one_candidates() {
    let problem = RaireProblem {
        metadata : json!({
            "candidates" : ["Alice","Bob","Chuan","Diego"]
        }),
        num_candidates : 1,
        votes : vec![],
        winner : None,
        audit : Audit::OneOnMargin(BallotComparisonOneOnDilutedMargin { total_auditable_ballots : BallotPaperCount(0) }),
        trim_algorithm: Some(TrimAlgorithm::MinimizeAssertions),
        difficulty_estimate: None,
        time_limit_seconds: None,
    };
    let solution = problem.solve().solution.unwrap();
    assert_eq!(CandidateIndex(0),solution.winner);
}



#[test]
/// Test 1 candidate with lots of votes, and 100 candidates with one vote each.
/// This checks the efficient computation of who won when lots of unimportant ties
/// exist.
fn test_efficient_who_wins() {
    let mut problem = RaireProblem {
        metadata : json!({
            "candidates" : ["Alice","Bob","Chuan","Diego"]
        }),
        num_candidates : 101,
        votes : vec![Vote{ n: BallotPaperCount(1000), prefs: vec![CandidateIndex(0)] }],
        winner : Some(CandidateIndex(0)),
        audit : Audit::OneOnMargin(BallotComparisonOneOnDilutedMargin { total_auditable_ballots : BallotPaperCount(1100) }),
        trim_algorithm: Some(TrimAlgorithm::MinimizeAssertions),
        difficulty_estimate: None,
        time_limit_seconds: Some(10.0), // Even on a very slow computer it shouldn't take a second to run. It takes 8ms on my four year old PC.
    };
    for i in 1..=100 {
        problem.votes.push(Vote{ n: BallotPaperCount(1), prefs: vec![CandidateIndex(i)] })
    }
    let solution = problem.solve().solution.unwrap();
    assert_eq!(CandidateIndex(0),solution.winner);
}


