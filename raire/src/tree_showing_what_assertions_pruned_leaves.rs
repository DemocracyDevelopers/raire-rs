// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.

#![doc = include_str!("../AssertionTrimmingAlgorithm.md")]

use std::cmp::Ordering;
use crate::assertions::{Assertion, AssertionAndDifficulty, EffectOfAssertionOnEliminationOrderSuffix};
use crate::irv::CandidateIndex;
use crate::raire_algorithm::TrimAlgorithm;
use crate::RaireError;
use crate::timeout::TimeOut;

/// Produce a tree of reverse-elimination-order descending down until either
/// * At least one assertion prunes all subsequent orders
/// * No assertions prune any subsequent order
///
/// One can optionally ask for an extended, which extends beyond pruned nodes if it is possible
/// for their children to be pruned. See HowFarToContinueSearchTreeWhenPruningAssertionFound for details.
/// This is useful for finding redundant assertions
/// that can be removed, at the cost of making the frontier larger.
pub struct TreeNodeShowingWhatAssertionsPrunedIt {
    pub candidate_being_eliminated_at_this_node: CandidateIndex, // The candidate eliminated at this step.
    pub pruning_assertions : Vec<usize>, // if any assertions prune it, their index in the main assertion list.
    pub children : Vec<TreeNodeShowingWhatAssertionsPrunedIt>, // its children, if any.
    pub valid : bool, // true if this node or a child thereof is not eliminated by any assertion.
}

impl TreeNodeShowingWhatAssertionsPrunedIt {
    /// Create a new tree node with a given path back to the root and candidate being eliminated.
    pub fn new (parent_elimination_order_suffix:&[CandidateIndex], candidate_being_eliminated_at_this_node:CandidateIndex, relevant_assertions:&[usize],all_assertions:&[Assertion],num_candidates:u32,consider_children_of_eliminated_nodes:HowFarToContinueSearchTreeWhenPruningAssertionFound,timeout:&mut TimeOut) -> Result<Self,RaireError> {
        if timeout.quick_check_timeout() { return Err(RaireError::TimeoutTrimmingAssertions) }
        let mut elimination_order_suffix=vec![candidate_being_eliminated_at_this_node]; // elimination order including this node
        elimination_order_suffix.extend_from_slice(parent_elimination_order_suffix);
        let mut pruning_assertions : Vec<usize> = vec![];
        let mut still_relevant_assertions : Vec<usize> = vec![];
        for &assertion_index in relevant_assertions {
            match all_assertions[assertion_index].ok_elimination_order_suffix(&elimination_order_suffix) {
                EffectOfAssertionOnEliminationOrderSuffix::Contradiction => { pruning_assertions.push(assertion_index); }
                EffectOfAssertionOnEliminationOrderSuffix::Ok => {} // can ignore
                EffectOfAssertionOnEliminationOrderSuffix::NeedsMoreDetail => { still_relevant_assertions.push(assertion_index); }
            }
        }
        let mut children : Vec<Self> = vec![];
        let mut valid : bool = pruning_assertions.is_empty() && still_relevant_assertions.is_empty();
        let pruned_by_neb = pruning_assertions.iter().any(|a|all_assertions[*a].is_neb());
        if (pruning_assertions.is_empty()||consider_children_of_eliminated_nodes.should_continue_if_pruning_assertion_found(pruned_by_neb)) && !still_relevant_assertions.is_empty() {
            let next_consider_children_of_eliminated_nodes = if pruning_assertions.is_empty() { consider_children_of_eliminated_nodes } else { consider_children_of_eliminated_nodes.next_level_if_pruning_assertion_found() };
            for candidate in 0..num_candidates {
                let candidate = CandidateIndex(candidate);
                if !elimination_order_suffix.contains(&candidate) { // could make more efficient by using binary search,
                    let child = TreeNodeShowingWhatAssertionsPrunedIt::new(&elimination_order_suffix,candidate,&still_relevant_assertions,all_assertions,num_candidates,next_consider_children_of_eliminated_nodes,timeout)?;
                    if child.valid {
                        if pruning_assertions.is_empty() {
                            valid=true;
                        } else { // we were continuing searching beyond a pruned branch. There is no point doing this.
                            children.clear();
                            break;
                        }
                    }
                    children.push(child);
                }
            }
        }
        Ok(TreeNodeShowingWhatAssertionsPrunedIt{candidate_being_eliminated_at_this_node,pruning_assertions,children,valid})
    }
}

#[derive(Copy, Clone,Debug)]
pub enum HowFarToContinueSearchTreeWhenPruningAssertionFound {
    /// When a pruning assertion is found, don't look any further. Minimizes size of pruning tree.
    StopImmediately,
    /// When a pruning assertion is found, continue and see if its descendents are sufficient to stop it. But once it is stopped by a frontier of descendents, don't try each of their descendents.
    ContinueOnce,
    /// When a pruning assertion is found, continue. Don't stop unless no assertions left.
    Forever,
    /// Like forever, but do stop at a pruning assertion if at least one NEB prunes it. This is a useful heuristic as in practice NEBs are almost never redundant but often have very large descendent trees that need searching.
    StopOnNEB,
}

impl HowFarToContinueSearchTreeWhenPruningAssertionFound {
    fn should_continue_if_pruning_assertion_found(self,pruned_by_neb:bool) -> bool {
        match self {
            Self::StopImmediately => false,
            Self::StopOnNEB => !pruned_by_neb,
            _ => true,
        }
    }
    fn next_level_if_pruning_assertion_found(self) -> Self {
        match self {
            Self::StopImmediately => Self::StopImmediately, // should never happen.
            Self::ContinueOnce => Self::StopImmediately,
            Self::Forever => Self::Forever,
            Self::StopOnNEB => Self::StopOnNEB,
        }
    }
}

/// When trimming, it is possible to also compute the boundary for the winning candidate,
/// and check that that is not trimmed. This is a nice consistency check. However, the
/// computation of the boundary for the winning candidate is often vastly more expensive than
/// the rest of the computation. So it is not enabled.
const CHECK_WINNER_NOT_ELIMINATED:bool=false;

/// Sort the assertions in a human sensible manner, and then trim them.
///
/// Note that if a timeout error is produced, the assertions array will be sorted but otherwise unchanged
/// from the original call.
///
/// The algorithm is described in [../AssertionTrimmingAlgorithm.md]
pub fn order_assertions_and_remove_unnecessary(assertions:&mut Vec<AssertionAndDifficulty>,winner:CandidateIndex,num_candidates:u32,trim_algorithm:TrimAlgorithm,timeout:&mut TimeOut) -> Result<(),RaireError> {
    assertions.sort_unstable_by(|a,b|{
        // sort all NEBs before NENs,
        // sort NENs by length
        // ties - sort by winner, then loser, then continuing
        match (&a.assertion,&b.assertion) {
            (Assertion::NEN(_), Assertion::NEB(_)) => Ordering::Greater,
            (Assertion::NEB(_), Assertion::NEN(_)) => Ordering::Less,
            (Assertion::NEN(a), Assertion::NEN(b)) => {
                a.continuing.len().cmp(&b.continuing.len()).then_with(||a.winner.0.cmp(&b.winner.0).then_with(||a.loser.0.cmp(&b.loser.0)).then_with(||{
                    // compare continuing
                    for i in 0..a.continuing.len() {
                        let res = a.continuing[i].0.cmp(&b.continuing[i].0);
                        if res!=Ordering::Equal { return res}
                    }
                    Ordering::Equal
                }))
            },
            (Assertion::NEB(a), Assertion::NEB(b)) => a.winner.0.cmp(&b.winner.0).then_with(||a.loser.0.cmp(&b.loser.0)),
        }
    });
    if let Some(consider_children_of_eliminated_nodes) = match trim_algorithm {
        TrimAlgorithm::None => None,
        TrimAlgorithm::MinimizeTree => Some(HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately),
        TrimAlgorithm::MinimizeAssertions => Some(HowFarToContinueSearchTreeWhenPruningAssertionFound::StopOnNEB),
    } { // do the actual trimming
        let all_assertions : Vec<Assertion> = assertions.iter().map(|ad|ad.assertion.clone()).collect();
        let all_assertion_indices : Vec<usize> = (0..all_assertions.len()).collect();
        let mut find_used = HeuristicWorkOutWhichAssertionsAreUsed::new(assertions.len());
        let mut trees = vec![];
        for candidate in 0..num_candidates { // create trees and do first pass
            let candidate = CandidateIndex(candidate);
            if candidate!=winner || CHECK_WINNER_NOT_ELIMINATED {
                let tree = TreeNodeShowingWhatAssertionsPrunedIt::new(&[],candidate,&all_assertion_indices,&all_assertions,num_candidates,consider_children_of_eliminated_nodes,timeout)?;
                if tree.valid!= (candidate==winner) { return Err(if candidate==winner { RaireError::InternalErrorRuledOutWinner} else { RaireError::InternalErrorDidntRuleOutLoser })}
                if candidate!=winner {
                    find_used.add_tree_forced(&tree);
                    trees.push(tree);
                }
            }
        }
        for tree in trees {
            find_used.add_tree_second_pass(&tree,timeout)?;
        }
        find_used.finish_second_pass()?;
        let mut res = vec![];
        for (index,a) in assertions.drain(..).enumerate() {
            if find_used.uses(index) { res.push(a); }
        }
        assertions.extend(res.drain(..));
        // println!(" Trimmed {} assertions down to {}",all_assertion_indices.len(),assertions.len());
    }
    Ok(())
}

/// A pretty simple method of computing which assertions are used which may not always
/// be optimal, but is fast, and, in practice, has turned out to be optimal for every case
/// I tried it on.
///
/// The general problem can be converted to a problem of selection at least one of a combination
/// of expressions. The heuristic is a first pass choosing ones where there is no choice, and
/// a second pass of choosing arbitrarily amongst the remaining ones where prior choices have
/// not solved it.
struct HeuristicWorkOutWhichAssertionsAreUsed {
    assertions_used : Vec<bool>,
}

impl HeuristicWorkOutWhichAssertionsAreUsed {
    fn new(len:usize) -> Self { Self{assertions_used:vec![false;len]}}
    fn uses(&self,index:usize) -> bool { self.assertions_used[index] }
    /// Some (most) nodes have exactly one assertion. Assign these assertions, as they MUST be used.
    fn add_tree_forced(&mut self,node:&TreeNodeShowingWhatAssertionsPrunedIt) {
        if node.pruning_assertions.len()>0 {
            //print!("{}",node.pruning_assertions.len());
            if node.children.is_empty() {
                if node.pruning_assertions.len()==1 { // must be used
                    self.assertions_used[node.pruning_assertions[0]]=true;
                }
            } else {
                //print!("*");
            }
        } else {
            for child in &node.children {
                self.add_tree_forced(child);
            }
        }
    }
    /// See if a node is already eliminated by the assertions marked as being used.
    fn node_already_eliminated(&self,node:&TreeNodeShowingWhatAssertionsPrunedIt) -> bool {
        let directly_eliminated = node.pruning_assertions.iter().any(|&v|self.assertions_used[v]); // one of the assertions eliminates the node.
        directly_eliminated || { // check to see if all the children are eliminated
            node.children.len()!=0 && node.children.iter().all(|c|self.node_already_eliminated(c))
        }
    }
    fn add_tree_second_pass(&mut self,node:&TreeNodeShowingWhatAssertionsPrunedIt,timeout:&mut TimeOut) -> Result<(),RaireError> {
        if timeout.quick_check_timeout() { return Err(RaireError::TimeoutTrimmingAssertions); }
        if node.pruning_assertions.len()>0 {
            //print!("{}",node.pruning_assertions.len());
            if !self.node_already_eliminated(node) { // not already solved by one assertion that rules out this node.
                // none already used. Simplistically take the first one.
                self.assertions_used[node.pruning_assertions[0]]=true;
            }
        } else {
            for child in &node.children {
                self.add_tree_second_pass(child,timeout)?;
            }
        }
        Ok(())
    }
    fn finish_second_pass(&self)  -> Result<(),RaireError> {Ok(())}
}

/*
use xdd::{BDDFactory, DecisionDiagramFactory, NodeIndex, NoMultiplicity, VariableIndex};
use std::collections::HashMap;

/// a more complex method of computing which assertions are used - just use the first from each list. Benefits: minimizes number of assertions. Drawbacks: often much slower, complex, requires dependencies.
/// This is not used as the simplistic method turns out to be optimal on all samples tested when the Forever option is used, and it is prohibitively slow when the ContinueOnce option is used.
struct OptimalWorkOutWhichAssertionsAreUsed {
    simple : SimplisticWorkOutWhichAssertionsAreUsed,
    factory : BDDFactory<u32,NoMultiplicity>,
    required : NodeIndex<u32,NoMultiplicity>,
    variables : GetXDDVariable,
}

impl OptimalWorkOutWhichAssertionsAreUsed {
    fn new(len:usize) -> Result<Self,RaireError> {
        if len>u16::MAX as usize { Err(RaireError::InternalErrorTrimming) }
        else {
            let variables = GetXDDVariable::new(len);
            Ok(Self{simple:SimplisticWorkOutWhichAssertionsAreUsed::new(len), factory: BDDFactory::new(variables.max_variable), required:NodeIndex::TRUE, variables })
        }
    }
    fn uses(&self,index:usize) -> bool { self.simple.uses(index) }
    /// Some (most) nodes have exactly one assertion. Assign these assertions, as they MUST be used.
    fn add_tree_forced(&mut self,node:&TreeNodeShowingWhatAssertionsPrunedIt) {
        self.simple.add_tree_forced(node);
    }
    /// compute an xdd function representing the constraints implied by the tree
    fn tree_to_xdd(&mut self,node:&TreeNodeShowingWhatAssertionsPrunedIt) -> NodeIndex<u32,NoMultiplicity> {
        if node.pruning_assertions.iter().any(|&a|self.uses(a)) {
            // short cut, nothing to do as the first pass dealt with it!
            print!("!");
            return NodeIndex::TRUE;
        }
        // compute an xdd function representing the constraints implied by the children of this node
        let children = if node.children.is_empty() { NodeIndex::FALSE} else {
            let mut res = NodeIndex::TRUE;
            for child in &node.children {
                let child_xdd = self.tree_to_xdd(child);
                res = self.factory.and(res,child_xdd);
            }
            res
        };
        if children.is_true() {// short cut, nothing to do as the first pass dealt with it via children!
            print!("^");
            return NodeIndex::TRUE;
        }
        // compute an xdd function representing the constraints implied by the pruning_assertions of this node
        let direct = {
            let mut res = NodeIndex::FALSE;
            for &a in &node.pruning_assertions {
                let variable = self.variables.variable(a);
                let a_xdd = self.factory.single_variable(variable);
                res = self.factory.or(res,a_xdd);
            }
            res
        };
        print!(".");
        self.factory.or(direct,children)
    }
    fn add_tree_second_pass(&mut self,node:&TreeNodeShowingWhatAssertionsPrunedIt) {
        let tree = self.tree_to_xdd(node);
        self.required=self.factory.and(tree,self.required);
        println!("xdd sub-size {}",self.factory.len());
        let renamer = self.factory.gc([self.required]);
        self.required=renamer.rename(self.required).expect("Lost main point");
        println!("xdd sub-size {}",self.factory.len());
    }
    fn finish_second_pass(&mut self) -> Result<(),RaireError> {
        println!("xdd size {}",self.factory.len());
        let solution = self.factory.find_satisfying_solution_with_minimum_number_of_variables(self.required).ok_or(RaireError::InternalErrorTrimming)?;
        for v in solution {
            let assertion = self.variables.decode(v);
            self.simple.assertions_used[assertion]=true;
        }
        Ok(())
    }

}

/// Order XDD variables in inverse order that they are received.
struct GetXDDVariable {
    max_variable : u16,
    next_variable : u16,
    variable_of_assertion : Vec<Option<VariableIndex>>,
    assertion_of_variable : Vec<usize>
}

impl GetXDDVariable {
    fn new(len:usize) -> Self {
        GetXDDVariable {
            max_variable: u16::MAX - 10, // The -10 is probably not needed.
            next_variable: u16::MAX - 10, // The -10 is probably not needed.
            variable_of_assertion: vec![None;len],
            assertion_of_variable: vec![],
        }
    }
    fn variable(&mut self,assertion:usize) -> VariableIndex {
        if let Some(v) = self.variable_of_assertion[assertion] {
            v
        } else {
            let v = VariableIndex(self.next_variable);
            self.next_variable-=1; // Should have some check against underflow.
            self.variable_of_assertion[assertion]=Some(v);
            self.assertion_of_variable.push(assertion);
            v
        }
    }
    fn decode(&self,variable:VariableIndex) -> usize {
        let index = (self.max_variable-variable.0) as usize;
        self.assertion_of_variable[index]
    }
    fn len(&self) -> u16 { self.max_variable-self.next_variable }
}
*/

#[cfg(test)]
mod tests {
    use crate::assertions::{Assertion, NotEliminatedBefore, NotEliminatedNext};
    use crate::irv::CandidateIndex;
    use crate::timeout::TimeOut;
    use crate::tree_showing_what_assertions_pruned_leaves::{HowFarToContinueSearchTreeWhenPruningAssertionFound, TreeNodeShowingWhatAssertionsPrunedIt};

    /// Get the assertions listed in "A guide to RAIRE".
    fn raire_guide_assertions() -> Vec<Assertion> {
        vec![
            Assertion::NEN(NotEliminatedNext{winner:CandidateIndex(0),loser:CandidateIndex(1),continuing:vec![CandidateIndex(0),CandidateIndex(1),CandidateIndex(2),CandidateIndex(3)]}),
            Assertion::NEN(NotEliminatedNext{winner:CandidateIndex(0),loser:CandidateIndex(3),continuing:vec![CandidateIndex(0),CandidateIndex(2),CandidateIndex(3)]}),
            Assertion::NEN(NotEliminatedNext{winner:CandidateIndex(2),loser:CandidateIndex(0),continuing:vec![CandidateIndex(0),CandidateIndex(2)]}),
            Assertion::NEN(NotEliminatedNext{winner:CandidateIndex(2),loser:CandidateIndex(3),continuing:vec![CandidateIndex(0),CandidateIndex(2),CandidateIndex(3)]}),
            Assertion::NEB(NotEliminatedBefore{winner:CandidateIndex(2),loser:CandidateIndex(1)}),
            Assertion::NEN(NotEliminatedNext{winner:CandidateIndex(0),loser:CandidateIndex(3),continuing:vec![CandidateIndex(0),CandidateIndex(3)]}),
        ]
    }

    #[test]
    fn it_works() {
        let all_assertions = raire_guide_assertions();
        let relevant_assertions : Vec<usize> = (0..all_assertions.len()).collect();
        let mut timeout = TimeOut::new(Some(1000),None);
        let mut timeout_instantly = TimeOut::new(Some(1),None);
        assert!(TreeNodeShowingWhatAssertionsPrunedIt::new(&[],CandidateIndex(0),&relevant_assertions,&all_assertions,4,HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately,&mut timeout_instantly).is_err());
        let tree0 = TreeNodeShowingWhatAssertionsPrunedIt::new(&[],CandidateIndex(0),&relevant_assertions,&all_assertions,4,HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately,&mut timeout).unwrap();
        let tree1 = TreeNodeShowingWhatAssertionsPrunedIt::new(&[],CandidateIndex(1),&relevant_assertions,&all_assertions,4,HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately,&mut timeout).unwrap();
        let tree2 = TreeNodeShowingWhatAssertionsPrunedIt::new(&[],CandidateIndex(2),&relevant_assertions,&all_assertions,4,HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately,&mut timeout).unwrap();
        let tree3 = TreeNodeShowingWhatAssertionsPrunedIt::new(&[],CandidateIndex(3),&relevant_assertions,&all_assertions,4,HowFarToContinueSearchTreeWhenPruningAssertionFound::StopImmediately,&mut timeout).unwrap();
        // check tree0 (candidate 0 elimination)
        assert_eq!(false,tree0.valid);
        assert_eq!(3,tree0.children.len());
        assert_eq!(vec![4],tree0.children[0].pruning_assertions);
        assert_eq!(vec![2],tree0.children[1].pruning_assertions);
        assert_eq!(0,tree0.children[2].pruning_assertions.len());
        assert_eq!(2,tree0.children[2].children.len());
        assert_eq!(vec![4],tree0.children[2].children[0].pruning_assertions);
        assert_eq!(vec![3],tree0.children[2].children[1].pruning_assertions);
        // check tree1
        assert_eq!(false,tree1.valid);
        assert_eq!(vec![4],tree1.pruning_assertions);
        // check tree2
        assert_eq!(true,tree2.valid); // candidate 2 won.
        // check tree3
        assert_eq!(false,tree3.valid);
        assert_eq!(3,tree3.children.len());
        assert_eq!(vec![5],tree3.children[0].pruning_assertions);
        assert_eq!(vec![4],tree3.children[1].pruning_assertions);
        assert_eq!(0,tree3.children[2].pruning_assertions.len());
        assert_eq!(2,tree3.children[2].children.len());
        assert_eq!(vec![1],tree3.children[2].children[0].pruning_assertions);
        assert_eq!(0,tree3.children[2].children[1].pruning_assertions.len());
        assert_eq!(vec![0],tree3.children[2].children[1].children[0].pruning_assertions);
    }
}
