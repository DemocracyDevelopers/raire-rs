// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! The types of assertions about the election, generally relative standings of various candidates


use crate::irv::{BallotPaperCount};
use serde::Deserialize;
use serde::Serialize;

pub type AssertionDifficulty = f64;

/// An audit type is a method for determining a difficulty (higher means more difficult, infinite means impossible) for
/// a comparison of two claimed tallys.
///
/// The active_paper_count should not be used, and is included for backwards compatability of some tests, and may soon be removed.
pub trait AuditType {
    fn difficulty(&self, lowest_tally_winner:BallotPaperCount, highest_tally_loser:BallotPaperCount, active_paper_count:BallotPaperCount) -> AssertionDifficulty;
}


/// A BRAVO ballot polling audit as described in the original paper.
#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
pub struct BallotPollingBRAVO {
    /// The desired confidence α. A number between 0 and 1 bounding the probability of not rejecting a false result.
    pub confidence : f64,
    pub total_auditable_ballots : BallotPaperCount,
}


impl BallotPollingBRAVO {
    /// compute ASN using the BRAVO method described in the original paper, except using only unexhausted ballot papers as the denominator computing p_w and p_l.
    /// Don't use this.
    pub fn average_sample_number_original_paper_using_active_paper_count(&self,lowest_tally_winner:BallotPaperCount,highest_tally_loser:BallotPaperCount,active_paper_count:BallotPaperCount) -> AssertionDifficulty {
        //println!("Doing BRAVO with winner {lowest_tally_winner} loser {highest_tally_loser} active_paper_count={active_paper_count} α={}",self.confidence);
        if lowest_tally_winner.0<=highest_tally_loser.0 { f64::INFINITY } else {
            let w = lowest_tally_winner.0 as f64;
            let l = highest_tally_loser.0 as f64;
            let s = w/(w+l);
            let twos = 2.0*s;
            let ln2s = twos.ln();
            let numerator = 0.5*ln2s-self.confidence.ln();
            let denominator = (w*ln2s+l*(2.0-twos).ln())/(active_paper_count.0 as f64);
            numerator/denominator
        }
    }
    /// compute ASN using the BRAVO method described in the original paper.
    pub fn average_sample_number_original_paper_using_total_auditable_ballots(&self,lowest_tally_winner:BallotPaperCount,highest_tally_loser:BallotPaperCount,_active_paper_count:BallotPaperCount) -> AssertionDifficulty {
        //println!("Doing BRAVO with winner {lowest_tally_winner} loser {highest_tally_loser} active_paper_count={active_paper_count} α={}",self.confidence);
        if lowest_tally_winner.0<=highest_tally_loser.0 { f64::INFINITY } else {
            let w = lowest_tally_winner.0 as f64;
            let l = highest_tally_loser.0 as f64;
            let s = w/(w+l);
            let twos = 2.0*s;
            let ln2s = twos.ln();
            let numerator = 0.5*ln2s-self.confidence.ln();
            let denominator = (w*ln2s+l*(2.0-twos).ln())/(self.total_auditable_ballots.0 as f64);
            numerator/denominator
        }
    }
}

impl AuditType for BallotPollingBRAVO {
    fn difficulty(&self, lowest_tally_winner: BallotPaperCount, highest_tally_loser: BallotPaperCount, active_paper_count:BallotPaperCount) -> AssertionDifficulty {
        self.average_sample_number_original_paper_using_total_auditable_ballots(lowest_tally_winner,highest_tally_loser,active_paper_count)
    }
}

/// like BallotPollingBRAVO but use the number of non-exhausted papers as the denominator in p_w and p_l.
pub struct BallotPollingBRAVOUsingActivePaperCount(pub BallotPollingBRAVO);

impl AuditType for BallotPollingBRAVOUsingActivePaperCount {
    fn difficulty(&self, lowest_tally_winner: BallotPaperCount, highest_tally_loser: BallotPaperCount, active_paper_count:BallotPaperCount) -> AssertionDifficulty {
        self.0.average_sample_number_original_paper_using_active_paper_count(lowest_tally_winner,highest_tally_loser,active_paper_count)
    }
}


/// A MACRO ballot level comparison audit.
#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
pub struct BallotComparisonMACRO {
    /// The desired confidence α. A number between 0 and 1 bounding the probability of not rejecting a false result.
    pub confidence : f64,
    /// γ ≥ 1
    pub error_inflation_factor : f64,
    pub total_auditable_ballots : BallotPaperCount,
}


impl BallotComparisonMACRO {
    /// Compute ASN using the
    pub fn average_sample_number_original_paper(&self,lowest_tally_winner:BallotPaperCount,highest_tally_loser:BallotPaperCount) -> AssertionDifficulty {
        if lowest_tally_winner<=highest_tally_loser { f64::INFINITY } else {
            let v = lowest_tally_winner-highest_tally_loser;
            let u = 2.0*self.error_inflation_factor*self.total_auditable_ballots.0 as f64/v.0 as f64;
            -self.confidence.ln()*u
        }
    }
}

impl AuditType for BallotComparisonMACRO {
    fn difficulty(&self, lowest_tally_winner: BallotPaperCount, highest_tally_loser: BallotPaperCount, _active_paper_count:BallotPaperCount) -> AssertionDifficulty {
        self.average_sample_number_original_paper(lowest_tally_winner,highest_tally_loser)
    }
}

/// A comparison where the difficulty = 1/diluted margin.
/// Useful for BallotComparison audits
#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
pub struct BallotComparisonOneOnDilutedMargin {
    pub total_auditable_ballots : BallotPaperCount,
}

impl AuditType for BallotComparisonOneOnDilutedMargin {
    fn difficulty(&self, lowest_tally_winner: BallotPaperCount, highest_tally_loser: BallotPaperCount, _active_paper_count:BallotPaperCount) -> AssertionDifficulty {
        if lowest_tally_winner<=highest_tally_loser { f64::INFINITY } else {
            let reciprocal_diluted_margin = self.total_auditable_ballots.0 as f64/(lowest_tally_winner-highest_tally_loser).0 as f64;
            reciprocal_diluted_margin
        }
    }
}


/// A comparison where the difficulty = 1/diluted margin^2.
/// Useful for Ballot Polling audits.
#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
pub struct BallotPollingOneOnDilutedMarginSquared {
    pub total_auditable_ballots : BallotPaperCount,
}

impl AuditType for BallotPollingOneOnDilutedMarginSquared {
    fn difficulty(&self, lowest_tally_winner: BallotPaperCount, highest_tally_loser: BallotPaperCount, _active_paper_count:BallotPaperCount) -> AssertionDifficulty {
        if lowest_tally_winner<=highest_tally_loser { f64::INFINITY } else {
            let reciprocal_diluted_margin = self.total_auditable_ballots.0 as f64/(lowest_tally_winner-highest_tally_loser).0 as f64;
            reciprocal_diluted_margin*reciprocal_diluted_margin
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(tag = "type")]
pub enum Audit {
    BRAVO(BallotPollingBRAVO),
    MACRO(BallotComparisonMACRO),
    #[serde(alias = "Margin")] // for backwards compatibility
    OneOnMargin(BallotComparisonOneOnDilutedMargin),
    #[serde(alias = "MarginSq")] // for backwards compatibility
    OneOnMarginSq(BallotPollingOneOnDilutedMarginSquared),
}

impl AuditType for Audit {
    fn difficulty(&self, lowest_tally_winner: BallotPaperCount, highest_tally_loser: BallotPaperCount, active_paper_count: BallotPaperCount) -> AssertionDifficulty {
        match self {
            Audit::BRAVO(audit) => audit.difficulty(lowest_tally_winner,highest_tally_loser,active_paper_count),
            Audit::MACRO(audit) => audit.difficulty(lowest_tally_winner,highest_tally_loser,active_paper_count),
            Audit::OneOnMargin(audit) => audit.difficulty(lowest_tally_winner, highest_tally_loser, active_paper_count),
            Audit::OneOnMarginSq(audit) => audit.difficulty(lowest_tally_winner, highest_tally_loser, active_paper_count),
        }
    }
}