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
use raire::irv::{BallotPaperCount, CandidateIndex};
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
        Err(RaireError::InvalidCandidateNumber) => {}
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


