// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


//! Test the examples from New South Wales, Australia, local government elections for Mayor 2021
//! (Mayoral ones chosen as they are IRV; others are STV).
//!
//! Note that this is not a great test for raire-rs
//! as it just tests that the results match prior results computed using raire-rs.
//! However this still tests against regressions, and the same data is used to test raire-java against raire-rs.
//!
//! Other tests compare hand crafted data to hand computed values and thus have independent confirmation. This is a regression test on large data.


use std::fs::File;
use std::path::PathBuf;
use raire::{RaireProblem, RaireSolution};

fn test_folder(folder:&str) {
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        // println!("Found {:?}",file_name);
        if file_name.ends_with(".json")&& !file_name.ends_with("_out.json") {
            println!("Processing {:?}",file_name);
            let problem : RaireProblem = serde_json::from_reader(File::open(&entry.path()).unwrap()).unwrap();
            let solution = problem.solve();
            let solution_file : PathBuf = PathBuf::from(folder).join(format!("{}_out.json",file_name.strip_suffix(".json").unwrap()));
            // The line below was originally used to create the expected output. See header comment about this being a poor test case.
            // serde_json::to_writer(File::create(&solution_file).unwrap(),&solution).unwrap();
            let expected_solution : RaireSolution = serde_json::from_reader(File::open(&solution_file).unwrap()).unwrap();
            // It is not an error to produce a different set of assertions with the same difficulty, so can only check difficulty.
            let expected_difficulty = expected_solution.solution.as_ref().unwrap().difficulty;
            let computed_difficulty = solution.solution.as_ref().unwrap().difficulty;
            println!("Expected difficulty for {} : {}, computed difficulty : {}",file_name,expected_difficulty,computed_difficulty);
            assert!((expected_difficulty-computed_difficulty).abs()<0.001);
        }
    }
}


#[test]
fn test_nsw_2021() {
    test_folder("../Australian Examples/NSW Local Government/2021/");
}