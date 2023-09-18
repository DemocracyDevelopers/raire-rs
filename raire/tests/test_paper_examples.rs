// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! Test the examples given in https://arxiv.org/pdf/1903.08804.pdf


use raire::assertions::{NotEliminatedNext, SpecificLoserAmongstContinuing, NotEliminatedBefore};
use raire::audit_type::{BallotComparisonMACRO, BallotPollingBRAVO};
use raire::irv::{BallotPaperCount, CandidateIndex, Vote, Votes};
use raire::raire_algorithm::{raire, TrimAlgorithm};

/// Get the votes in table 1.
fn get_votes() -> Votes {
    let c1 = CandidateIndex(0);
    let c2 = CandidateIndex(1);
    let c3 = CandidateIndex(2);
    let c4 = CandidateIndex(3);
    let votes = vec![
        Vote{ n: BallotPaperCount( 4000), prefs: vec![c2,c3]},
        Vote{ n: BallotPaperCount(20000), prefs: vec![c1]},
        Vote{ n: BallotPaperCount( 9000), prefs: vec![c3,c4]},
        Vote{ n: BallotPaperCount( 6000), prefs: vec![c2,c3,c4]},
        Vote{ n: BallotPaperCount(15000), prefs: vec![c4,c1,c2]},
        Vote{ n: BallotPaperCount( 6000), prefs: vec![c1,c3]},
    ];
    Votes::new(votes, 4)
}

/// Get the votes in table 1.
fn get_votes_for_example5() -> Votes {
    let c1 = CandidateIndex(0);
    let c2 = CandidateIndex(1);
    let c3 = CandidateIndex(2);
    let c4 = CandidateIndex(3);
    let c5 = CandidateIndex(4);
    let votes = vec![
        Vote{ n: BallotPaperCount(10000), prefs: vec![c1]},
        Vote{ n: BallotPaperCount( 6000), prefs: vec![c2]},
        Vote{ n: BallotPaperCount( 3000), prefs: vec![c3,c2]},
        Vote{ n: BallotPaperCount( 2000), prefs: vec![c3,c1]},
        Vote{ n: BallotPaperCount(  500), prefs: vec![c4]},
        Vote{ n: BallotPaperCount(  499), prefs: vec![c5]},
    ];
    Votes::new(votes, 5)
}


/// Get the votes for the election in example 9
fn get_votes_for_example9() -> Votes {
    let c1 = CandidateIndex(0);
    let c2 = CandidateIndex(1);
    let c3 = CandidateIndex(2);
    let votes = vec![
        Vote{ n: BallotPaperCount(10000), prefs: vec![c1,c2,c3]},
        Vote{ n: BallotPaperCount( 6000), prefs: vec![c2,c1,c3]},
        Vote{ n: BallotPaperCount( 5999), prefs: vec![c3,c1,c2]},
    ];
    Votes::new(votes, 3)
}


/// Get the votes for the election in example 12
fn get_votes_for_example12() -> Votes {
    let c1 = CandidateIndex(0);
    let c2 = CandidateIndex(1);
    let c3 = CandidateIndex(2);
    let c4 = CandidateIndex(3);
    let votes = vec![
        Vote{ n: BallotPaperCount(5000), prefs: vec![c1,c2,c3]},
        Vote{ n: BallotPaperCount(5000), prefs: vec![c1,c3,c2]},
        Vote{ n: BallotPaperCount(5000), prefs: vec![c2,c3,c1]},
        Vote{ n: BallotPaperCount(1500), prefs: vec![c2,c1,c3]},
        Vote{ n: BallotPaperCount(5000), prefs: vec![c3,c2,c1]},
        Vote{ n: BallotPaperCount( 500), prefs: vec![c3,c1,c1]},
        Vote{ n: BallotPaperCount(5000), prefs: vec![c4,c1]},
    ];
    Votes::new(votes, 4)
}


// const BRAVO_EG1: BallotPollingBRAVOUsingActivePaperCount = BallotPollingBRAVOUsingActivePaperCount(BallotPollingBRAVO{ confidence: 0.05, total_auditable_ballots: BallotPaperCount(60000) }); // This is what is needed to match the paper
const BRAVO_EG1: BallotPollingBRAVO = BallotPollingBRAVO{ confidence: 0.05, total_auditable_ballots: BallotPaperCount(60000) }; // This is what I think it should be
const MACRO : BallotComparisonMACRO = BallotComparisonMACRO{ confidence: 0.05, error_inflation_factor: 1.1, total_auditable_ballots: BallotPaperCount(60000) };

// also works for 9.
// const BRAVO_EG5 : BallotPollingBRAVOUsingActivePaperCount = BallotPollingBRAVOUsingActivePaperCount(BallotPollingBRAVO{ confidence: 0.05, total_auditable_ballots: BallotPaperCount(21999) }); // This is what is needed to match the paper
const BRAVO_EG5 : BallotPollingBRAVO = BallotPollingBRAVO{ confidence: 0.05, total_auditable_ballots: BallotPaperCount(21999) }; // This is what I think it should be
const MACRO_EG5 : BallotComparisonMACRO = BallotComparisonMACRO{ confidence: 0.05, error_inflation_factor: 1.1, total_auditable_ballots: BallotPaperCount(21999) };

const BRAVO_EG12 : BallotPollingBRAVO = BallotPollingBRAVO{ confidence: 0.05, total_auditable_ballots: BallotPaperCount(27000) };
const MACRO_EG12 : BallotComparisonMACRO = BallotComparisonMACRO{ confidence: 0.05, error_inflation_factor: 1.1, total_auditable_ballots: BallotPaperCount(27000) };


#[test]
/// Test the get_votes() function and the methods on the Votes object.
fn test_votes_structure() {
    let votes = get_votes();
    assert_eq!(BallotPaperCount(60000),votes.total_votes());
    assert_eq!(BallotPaperCount(26000),votes.first_preference_only_tally(CandidateIndex(0)));
    assert_eq!(BallotPaperCount(10000),votes.first_preference_only_tally(CandidateIndex(1)));
    assert_eq!(BallotPaperCount( 9000),votes.first_preference_only_tally(CandidateIndex(2)));
    assert_eq!(BallotPaperCount(15000),votes.first_preference_only_tally(CandidateIndex(3)));
    assert_eq!(vec![BallotPaperCount(26000),BallotPaperCount(10000),BallotPaperCount(24000)],votes.restricted_tallies(&vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(3)]));
    assert_eq!(vec![BallotPaperCount(26000),BallotPaperCount(30000)],votes.restricted_tallies(&vec![CandidateIndex(0),CandidateIndex(3)]));
}

#[test]
/// Check the final ASN for example 2 in the paper
fn test_example2() {
    let votes = get_votes();
    let assertion = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3)], losers: vec![CandidateIndex(2)] };
    let asn = assertion.difficulty(&votes, &BRAVO_EG1);
    println!("Example 2 : ASN={}",asn);
    assert!((asn-6885.0).abs()<1.0);
}


#[test]
/// Check the final ASN for example 3 in the paper
fn test_example3() {
    let votes = get_votes();
    let assertion = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3)], losers: vec![CandidateIndex(2)] };
    let asn = assertion.difficulty(&votes, &MACRO);
    println!("Example 3 : ASN={}",asn);
    assert!((asn-395.4).abs()<0.1);
}

#[test]
/// Check the ASNs for example 4 in the paper
fn test_example4() {
    let votes = get_votes();
    let assertion1 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3)], losers: vec![CandidateIndex(2)] };
    let assertion2 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(3)], losers: vec![CandidateIndex(1)] };
    let assertion3 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(3)], losers: vec![CandidateIndex(0)] };
    let asn1 = assertion1.difficulty(&votes, &BRAVO_EG1);
    let asn2 = assertion2.difficulty(&votes, &BRAVO_EG1);
    let asn3 = assertion3.difficulty(&votes, &BRAVO_EG1);
    println!("Example 4 : ASN={asn1} for exclusion 1, {asn2} for exclusion 2 and {asn3} for exclusion 3");
    assert!((asn1-6885.0).abs()<1.0);
    assert!((asn2-64.0).abs()<0.1);
    assert!((asn3-1271.6).abs()<1.0);
    // what happens if you only count the continuing ballots
    // TODO fix paper draft
    let asn3 = BRAVO_EG1.bravo_function(BallotPaperCount(30000),BallotPaperCount(26000),BallotPaperCount(56000));
    assert!((asn3-1186.0).abs()<1.0);
}

#[test]
/// Check the ASN for example 5 in the paper
fn test_example5() {
    let votes = get_votes_for_example5();
    assert_eq!(BRAVO_EG5.total_auditable_ballots,votes.total_votes());
    let assertion = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3),CandidateIndex(4)], losers: vec![CandidateIndex(4)] };
    let asn = assertion.difficulty(&votes, &BRAVO_EG5);
    println!("Example 5 : ASN={}",asn);
    // TODO I get 131 696 388 which is an order of magnitude higher than the value 13 165 239 in the paper. I have my value in the assertion below.
    assert!((asn-131696388.0).abs()<1.0);
}


#[test]
/// Check the ASNs for example 4 in the paper
fn test_example6() {
    let votes = get_votes();
    let assertion1 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3)], losers: vec![CandidateIndex(2)] };
    let assertion2 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(3)], losers: vec![CandidateIndex(1)] };
    let assertion3 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(3)], losers: vec![CandidateIndex(0)] };
    let asn1 = assertion1.difficulty(&votes, &MACRO);
    let asn2 = assertion2.difficulty(&votes, &MACRO);
    let asn3 = assertion3.difficulty(&votes, &MACRO);
    println!("Example 6 : ASN={asn1} for exclusion 1, {asn2} for exclusion 2 and {asn3} for exclusion 3");
    assert!((asn1-395.4).abs()<0.1);
    assert!((asn2-28.2).abs()<0.1);
    assert!((asn3-98.9).abs()<0.1);
}


#[test]
/// Check the ASN for example 7 in the paper
fn test_example7() {
    let votes = get_votes_for_example5();
    assert_eq!(BRAVO_EG5.total_auditable_ballots,votes.total_votes());
    let assertion1 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3),CandidateIndex(4)], losers: vec![CandidateIndex(3),CandidateIndex(4)] };
    let assertion2 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2)], losers: vec![CandidateIndex(2)] };
    let assertion3 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3),CandidateIndex(4)], losers: vec![CandidateIndex(2),CandidateIndex(3),CandidateIndex(4)] };
    let asn1 = assertion1.difficulty(&votes, &BRAVO_EG5);
    let asn2 = assertion2.difficulty(&votes, &BRAVO_EG5);
    let asn3 = assertion3.difficulty(&votes, &BRAVO_EG5);
    println!("Example 7 : ASN1={asn1} ASN2={asn2} ASN3={asn3}");
    assert!((asn1-49.1).abs()<0.1);
    assert!((asn2-1468.89).abs()<0.01);
    // TODO this is off by an order of magnitude. There is an extra digit "2". compared to the paper 158156493
    assert!((asn3-1581564932.0).abs()<1.0);
    // what happens if you only count the continuing ballots
    // TODO fix paper draft
    let asn2 = BRAVO_EG1.bravo_function(BallotPaperCount(6000),BallotPaperCount(5000),BallotPaperCount(21000));
    assert!((asn2-1402.0).abs()<1.0);
}


#[test]
/// Check the ASN for example 8 in the paper
fn test_example8() {
    let votes = get_votes_for_example5();
    assert_eq!(MACRO_EG5.total_auditable_ballots,votes.total_votes());
    let assertion1 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3),CandidateIndex(4)], losers: vec![CandidateIndex(3),CandidateIndex(4)] };
    let assertion2 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2)], losers: vec![CandidateIndex(2)] };
    let assertion3 = SpecificLoserAmongstContinuing{ continuing: vec![CandidateIndex(0),CandidateIndex(1)], losers: vec![CandidateIndex(1)] };
    let asn1 = assertion1.difficulty(&votes, &MACRO_EG5);
    let asn2 = assertion2.difficulty(&votes, &MACRO_EG5);
    let asn3 = assertion3.difficulty(&votes, &MACRO_EG5);
    println!("Example 7 : ASN1={asn1} ASN2={asn2} ASN3={asn3}");
    assert!((asn1-36.2).abs()<0.1);
    assert!((asn2-145.0).abs()<1.0);
    assert!((asn3-48.3).abs()<0.1);
}


#[test]
/// Check the ASN for example 10 in the paper
fn test_example10() {
    let votes = get_votes_for_example9();
    assert_eq!(BRAVO_EG5.total_auditable_ballots,votes.total_votes());
    let assertion1 = NotEliminatedBefore { winner:CandidateIndex(0), loser: CandidateIndex(1) };
    let assertion2 = NotEliminatedBefore { winner:CandidateIndex(0), loser: CandidateIndex(2) };
    let asn1 = assertion1.difficulty(&votes, &BRAVO_EG5).0;
    let asn2 = assertion2.difficulty(&votes, &BRAVO_EG5).0;
    println!("Example 7 : ASN1={asn1} ASN2={asn2} ");
    assert!((asn1-135.3).abs()<0.1);
    assert!((asn2-135.2).abs()<0.1);
    // what happens if you only count the continuing ballots
    // TODO fix paper draft
    let asn1 = BRAVO_EG1.bravo_function(BallotPaperCount(10000),BallotPaperCount(6000),BallotPaperCount(16000));
    let asn2 = BRAVO_EG1.bravo_function(BallotPaperCount(10000),BallotPaperCount(5999),BallotPaperCount(15999));
    println!("Example 7 using only continuing ballots : ASN1={asn1} ASN2={asn2} ");
    assert!((asn1-98.4).abs()<0.1);
    assert!((asn2-98.3).abs()<0.1);

}


#[test]
/// Check the ASN for example 11 in the paper
fn test_example11() {
    let votes = get_votes_for_example9();
    assert_eq!(MACRO_EG5.total_auditable_ballots,votes.total_votes());
    let assertion1 = NotEliminatedBefore { winner:CandidateIndex(0), loser: CandidateIndex(1) };
    let assertion2 = NotEliminatedBefore { winner:CandidateIndex(0), loser: CandidateIndex(2) };
    let asn1 = assertion1.difficulty(&votes, &MACRO_EG5).0;
    let asn2 = assertion2.difficulty(&votes, &MACRO_EG5).0;
    println!("Example 7 : ASN1={asn1} ASN2={asn2} ");
    assert!((asn1-36.2).abs()<0.1);
    assert!((asn2-36.2).abs()<0.1);
}


#[test]
/// Check the ASN for example 12 in the paper
fn test_example12_asns() {
    let votes = get_votes_for_example12();
    assert_eq!(BRAVO_EG12.total_auditable_ballots,votes.total_votes());
    assert_eq!(MACRO_EG12.total_auditable_ballots,votes.total_votes());
    // ballot polling
    let assertion1 = NotEliminatedNext { winner:CandidateIndex(0), loser: CandidateIndex(1), continuing: vec![CandidateIndex(0), CandidateIndex(1)] };
    let assertion2 = NotEliminatedNext { winner:CandidateIndex(0), loser: CandidateIndex(2), continuing: vec![CandidateIndex(0), CandidateIndex(2)] };
    let assertion3 = NotEliminatedBefore { winner:CandidateIndex(0), loser: CandidateIndex(3) };
    let assertion4 = NotEliminatedNext { winner:CandidateIndex(0), loser: CandidateIndex(2), continuing: vec![CandidateIndex(0), CandidateIndex(1), CandidateIndex(2)] };
    let asn1 = assertion1.difficulty(&votes, &BRAVO_EG12);
    let asn2 = assertion2.difficulty(&votes, &BRAVO_EG12);
    let asn3 = assertion3.difficulty(&votes, &BRAVO_EG12).0;
    let asn4 = assertion4.difficulty(&votes, &BRAVO_EG12);
    println!("Example 7 : ASN1={asn1} ASN2={asn2}  ASN3={asn3}  ASN4={asn4}");
    let asn1p = 100.0*asn1/votes.total_votes().0 as f64;
    let asn2p = 100.0*asn2/votes.total_votes().0 as f64;
    let asn3p = 100.0*asn3/votes.total_votes().0 as f64;
    let asn4p = 100.0*asn4/votes.total_votes().0 as f64;
    println!("Example 7 percentages : ASN1={asn1p}% ASN2={asn2p}%  ASN3={asn3p}%  ASN4={asn4p}%");
    assert!((asn1p-1.0).abs()<0.1);
    assert!((asn2p-0.5).abs()<0.1);
    assert!((asn3p-0.4).abs()<0.1);
    assert!((asn4p-0.1).abs()<0.1);
    // ballot comparison
    let assertion1 = NotEliminatedNext { winner:CandidateIndex(0), loser: CandidateIndex(1), continuing: vec![CandidateIndex(0), CandidateIndex(1)] };
    let assertion2 = NotEliminatedNext { winner:CandidateIndex(0), loser: CandidateIndex(2), continuing: vec![CandidateIndex(0), CandidateIndex(1), CandidateIndex(2)] };
    let assertion3 = NotEliminatedNext { winner:CandidateIndex(0), loser: CandidateIndex(2), continuing: vec![CandidateIndex(0), CandidateIndex(2)] };
    let assertion4 = NotEliminatedBefore { winner:CandidateIndex(0), loser: CandidateIndex(3) };
    let assertion5a = NotEliminatedNext { winner:CandidateIndex(1), loser: CandidateIndex(3), continuing: vec![CandidateIndex(1), CandidateIndex(3)] };
    let assertion5b = NotEliminatedNext { winner:CandidateIndex(2), loser: CandidateIndex(3), continuing: vec![CandidateIndex(2), CandidateIndex(3)] };
    let asn1 = assertion1.difficulty(&votes, &MACRO_EG12);
    let asn2 = assertion2.difficulty(&votes, &MACRO_EG12);
    let asn3 = assertion3.difficulty(&votes, &MACRO_EG12);
    let asn4 = assertion4.difficulty(&votes, &MACRO_EG12).0;
    let asn5a = assertion5a.difficulty(&votes, &MACRO_EG12);
    let asn5b = assertion5b.difficulty(&votes, &MACRO_EG12);
    println!("Example 7 : ASN1={asn1} ASN2={asn2}  ASN3={asn3}  ASN4={asn4} ASN5={asn5a} and {asn5b}");
    let asn1p = 100.0*asn1/votes.total_votes().0 as f64;
    let asn2p = 100.0*asn2/votes.total_votes().0 as f64;
    let asn3p = 100.0*asn3/votes.total_votes().0 as f64;
    let asn4p = 100.0*asn4/votes.total_votes().0 as f64;
    let asn5pa = 100.0*asn5a/votes.total_votes().0 as f64;
    let asn5pb = 100.0*asn5b/votes.total_votes().0 as f64;
    println!("Example 7 percentages : ASN1={asn1p}% ASN2={asn2p}%  ASN3={asn3p}%  ASN4={asn4p}% ASN5={asn5pa}% and {asn5pb}%");
    assert!((asn1p-0.17).abs()<0.01);
    assert!((asn2p-0.07).abs()<0.01);
    assert!((asn3p-0.11).abs()<0.01);
    assert!((asn4p-0.13).abs()<0.01);
    assert!((asn5pa-0.04).abs()<0.01);
    assert!((asn5pb-0.04).abs()<0.01);
}

/// Test that RAIRE produces reasonable answers for the BRAVO audit type.
#[test]
fn test_example12_raire_bravo() {
    let votes = get_votes_for_example12();
    assert_eq!(BRAVO_EG12.total_auditable_ballots, votes.total_votes());
    let res = raire(&votes,Some(CandidateIndex(0)),&BRAVO_EG12,TrimAlgorithm::None).unwrap();
    println!("{:?}",res);
    assert!((res.difficulty -278.25).abs()<0.01);
    let elimination_orders = res.possible_elimination_orders_allowed_by_assertions(votes.num_candidates());
    println!("Allowed elimination orders : {:?}", elimination_orders);
    assert_ne!(0, elimination_orders.len());
    for e in elimination_orders {
        let winning_candidate = e.last().cloned();
        assert_eq!(Some(CandidateIndex(0)),winning_candidate);
    }
    let elimination_suffixes = res.possible_elimination_order_suffixes_allowed_by_assertions(votes.num_candidates());
    println!("Allowed elimination order suffixes : {:?}", elimination_suffixes);
    assert_ne!(0, elimination_suffixes.len());
    for e in elimination_suffixes {
        let winning_candidate = e.last().cloned();
        assert_eq!(Some(CandidateIndex(0)),winning_candidate);
    }
}

/// Test that RAIRE produces reasonable answers for the MACRO audit type.
#[test]
fn test_example12_raire_macro() {
    let votes = get_votes_for_example12();
    assert_eq!(MACRO_EG12.total_auditable_ballots, votes.total_votes());
    let res = raire(&votes,Some(CandidateIndex(0)),&MACRO_EG12,TrimAlgorithm::None).unwrap();
    println!("{:?}",res);
    assert!((res.difficulty -44.49).abs()<0.01);
    let elimination_orders = res.possible_elimination_orders_allowed_by_assertions(votes.num_candidates());
    println!("Allowed elimination orders : {:?}", elimination_orders);
    assert_ne!(0, elimination_orders.len());
    for e in elimination_orders {
        let winning_candidate = e.last().cloned();
        assert_eq!(Some(CandidateIndex(0)),winning_candidate);
    }
    let elimination_suffixes = res.possible_elimination_order_suffixes_allowed_by_assertions(votes.num_candidates());
    println!("Allowed elimination order suffixes : {:?}", elimination_suffixes);
    assert_ne!(0, elimination_suffixes.len());
    for e in elimination_suffixes {
        let winning_candidate = e.last().cloned();
        assert_eq!(Some(CandidateIndex(0)),winning_candidate);
    }
}