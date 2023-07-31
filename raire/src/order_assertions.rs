// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! Order assertions to remove as much as possible first.



use crate::assertions::{AssertionAndDifficulty, EliminationOrderSuffix};
use crate::irv::CandidateIndex;

/// Change the list of assertions to order them with the first removing the most undesired elimination orders,
/// the second removing the most of what is left, etc.
///
/// Assertions that don't remove anything other than from places where the winner ends will be removed.
///
pub fn order_assertions_and_remove_unnecessary(assertions:&mut Vec<AssertionAndDifficulty>,winner:CandidateIndex,num_candidates:u32) {
    // scores[i] = the number of elimination orders implied by a particular prefix length.
    // This is done as f64 as the numbers can get large for a large number of candidates, and a rounding error is not so bad... at worst it will lead to a non-ideal order.
    let mut scores : Vec<f64> = vec![0.0;(num_candidates+1) as usize];
    scores[num_candidates as usize]=1.0;
    for i in 1..=num_candidates {
        scores[(num_candidates-i) as usize]=(i as f64)*scores[(num_candidates-i+1) as usize];
    }
    let mut elimination_orders : Vec<EliminationOrderSuffix> = (0..num_candidates).into_iter().filter(|c|*c!=winner.0).map(|c|vec![CandidateIndex(c)]).collect();
    let mut upto = 0; // assertions[0..upto] are dealt with.
    while upto<assertions.len() {
        println!("Current elimination orders {upto}:");
        for e in &elimination_orders { println!("{:?}",e); }
        // see if the remaining assertions have any point
        if elimination_orders.len()==0 { // we are done! remaining things are useless.
            assertions.truncate(upto);
            break;
        }
        // make sure assertions[upto] is the best of the remaing
        let mut best_improvement : f64 = 0.0;
        for j in upto..assertions.len() {
            let assertion = &assertions[j].assertion;
            let mut improvement : f64 = 0.0;
            for before in &elimination_orders {
                improvement+=scores[before.len()];
                for after in assertion.allowed_suffixes(before.clone(),num_candidates) {
                    improvement-=scores[after.len()];
                }
            }
            if improvement>best_improvement {
                best_improvement=improvement;
                if j!=upto {
                    assertions.swap(j,upto)
                }
            }
        }
        // update elimination orders
        let mut new_elimination_orders = vec![];
        for before in elimination_orders.drain(..) {
            new_elimination_orders.append(&mut assertions[upto].assertion.allowed_suffixes(before,num_candidates));
        }
        elimination_orders=new_elimination_orders;
        upto+=1;
    }
}
