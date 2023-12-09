# RAIRE-rs

This is a Rust port of RAIRE,
based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
documented in https://arxiv.org/pdf/1903.08804.pdf

This is a program designed to help with performing a risk limiting audit of an IRV election
by choosing a set of assertions, each of which can be audited, and which together imply that
the winner of the election was the expected candidate.

Note that there will not always exist such a set of assertions. For instance, if the contest
were a tie, then there is no way that a sample of votes could provide evidence the result
is correct as a single vote difference would be enough to change the outcome. However experiments
show that in many real elections it is possible to find such assertions and perform a risk
limiting audit.

# How to compile

ConcreteSTV is written in [Rust](https://www.rust-lang.org/). Install Rust (latest stable version
recommended), then run, in this directory,
```bash
cargo build --release
```

This will create several binary programs in the `target/release` directory.

# How to run as a command line program

There is a command line program called `raire` that takes an input JSON
file containing commands and produces and writes its output to another JSON
file. Run it as follows (change path names for different data)

```bash
./target/release/raire WebContent/example_input/a_guide_to_RAIRE_eg_guide.json out.json
```

This will make a file `out.json` in the current directory listing the assertions.

# How to run as a web service

There is a command line program called `raire-webserver` that 
listens at [http://localhost:3000/raire](http://localhost:3000/raire) for a
POST request providing the input JSON and returning the output JSON.

There is a human readable demo also provided at [http://localhost:3000/](http://localhost:3000/)
demonstrating the use of the API and the interpretation of the result.
There is also a human readable interpretation of the output of a variety
of formats of RAIRE outputs at [http://localhost:3000/explain_assertions.html](http://localhost:3000/explain_assertions.html).


```bash
./target/release/raire-webserver
```

# JSON input format

See examples in [WebContent/example_assertions](WebContent/example_assertions) for some examples taken from "A guide to RAIRE".

Here is a simple example for a contest with four candidates, Alice, Bob, Chuan and Diego. The winner was Chuan. There were 13500 ballots
of which there were 5000 putting Chuan first, then Bob, then Alice. There were 1000 listing Bob, then Chuan, then Diego. 
There were 1500 listing Diego then Alice. There were 4000 listing Alice then Diego, and 2000 listing just Diego. The audit
type is "OneOnMargin" with a total number of auditable ballots of 13500.
```text
{
  "metadata": {
    "candidates": ["Alice", "Bob", "Chuan","Diego" ],
    "note" : "Anything can go in the metadata section. Candidates names are used below if present. "
  },
  "num_candidates": 4,
  "votes": [
    { "n": 5000, "prefs": [ 2, 1, 0 ] },
    { "n": 1000, "prefs": [ 1, 2, 3 ] },
    { "n": 1500, "prefs": [ 3, 0 ] },
    { "n": 4000, "prefs": [ 0, 3 ] },
    { "n": 2000, "prefs": [ 3 ]  }
  ],
  "winner": 2,
  "audit": { "type": "OneOnMargin", "total_auditable_ballots": 13500  }
}
```

The input is JSON, with a single object containing the following fields:
* `metadata` : An arbitrary JSON object. The following sub-fields are used in some of the associated tools: 
  * `candidates` : An array of candidate names (one string for each candidate). The length of this array should match the *num_candidates* field.
  * `contest` : If present, the name of the contest (a string)
* `num_candidates` : An integer specifying how many candidates there are in the contest
* `votes` : An array of objects. Each object contains two fields:
  * `n` : The number of votes with this specific preference list
  * `prefs` : An array of integers between 0 and _num_candidates_-1, being indices of candidates in the preference list, with the most preferred candidate the first entry in the array.
* `winner` : Optionally, an integer between 0 and _num_candidates_-1, being the index of the candidate who is the winner. This will be checked against the votes as a consistency check.
  The only use for this is consistency checking - the RAIRE algorithm will recount the election anyway and check the winner. This is to prevent the audit checking that the digitally recorded
  votes do give the same winner as the paper ballots, but failing to notice that that is not the announced winner.
* `audit` : The type of the audit, and the number of auditable ballots for computing the diluted margin, which may be larger than the number of formal votes for a variety of logistic reasons. Audit type may be:
  * `BRAVO` : other parameter `"total_auditable_ballots"` and `"confidence"`,
  * `MACRO` : other parameters `"total_auditable_ballots"`, `"confidence"` and `error_inflation_factor` (a.k.a. γ),
  * `OneOnMargin` : other parameter `"total_auditable_ballots"`,
  * `OneOnMarginSq` : other parameter `"total_auditable_ballots"`
  These have various relevant parameters - see [the code](raire/src/audit_type.rs) for details. For example, for a generic ballot-level comparison audit, the appropriate type is
    `"audit": { "type": "OneOnMargin", "total_auditable_ballots": 42 }`
  where '42' is replaced by the appropriate number of ballots.
* `trim_algorithm`: Optionally one of the following strings : `None`, `MinimizeTree` (default if left blank), or `MinimizeAssertions`. The RAIRE algorithm may produce redundant assertions; there is a post-processing
  step that will trim redundant assertions. It will not change the difficulty score of the audit, but may reduce the number of assertions that need to be tested.
  * `"None"` does no such post-processing. 
  * `"MinimizeTree"` does minimal post-processing designed to minimize the total size of the tree showing all possible elimination orders until they are ruled out by an assertion. This is almost always quite fast, and a safe option which probably minimizes human effort to verify the output.
  * `"MinimizeAssertions"` does more complex post-processing that can eliminate more redundant assertions, at the expense of a possibly larger tree of possible elimination audits. This is often fast, but can sometimes take significantly longer than the main RAIRE algorithm.
* `difficulty_estimate` Optional (and you probably don't want to use it) number, an estimate of the difficulty. 
  If you know the difficulty in advance (by some magic or wild guess), you
  can set this number, and it will take it to be a lower bound on the difficulty of the problem. This could potentially make the algorithm 
  faster. This is probably not useful in practice, but is useful for performance testing and algorithm experimentation. In practice, the
  heuristics seem to usually do a good enough job of finding the optimum value that this doesn't help much even if you have a magic oracle.
* `time_limit_seconds` : Optional positive number limiting the number of seconds that are spent on the algorithm. This time will be somewhat infrequently checked,
  so don't expect this to be accurate to milliseconds.

# JSON output format

The output is JSON with two fields:
* `metadata` : a copy of the input metadata
* `solution` : An object with exactly one of the two following fields
  * `Err` : If some error occurred. Complete list of possibilities in [enum RaireError](raire/src/lib.rs)
  * `Ok` : If no error occurred. Value is a structure with the following fields:
    * `assertions` : an array of assertions. Each of these is an object with the following fields
      * `assertion` : on object containing fields
        * `type` : either the string `NEN` or `NEB` specifying what type of assertion it is.
          * `NEB` (Not Eliminated Before) means that the `winner` always beats the `loser`.
          * `NEN` (Not Eliminated Next) means that the `winner` beats the `loser` at the point where exactly the `continuing` candidates are remaining. In particular, it means that the `winner` is not eliminated at that exact point. 
        * `winner` : A candidate index
        * `loser` : A candidate index.
        * `continuing` : Only present if `type` is `NEN`. An array of candidate indices.
      * `difficulty` : a number indicating the difficulty of the assertion.
      * `margin` : an integer indicating the difference in the tallies associated with the winner and loser.
    * `difficulty` : a number indicating the difficulty of the audit. This is the maximum of the difficulties in the assertions array.
    * `margin` : an integer indicating the smallest margin of the audit. This is the minimum of the margins in the assertions array.
    * `winner` : The index of the candidate who won - an integer between `0` and `num_candidates-1`. 
    * `num_candidates` : The number of candidates (an integer).
    * `warning_trim_timed_out` : If present (and true), then the algorithm successfully found some assertions but was unable
      to do the desired trimming in the time limit provided. Instead the untrimmed assertions are returned. Some of them
      may be redundant.
    * `time_to_determine_winners`, `time_to_find_assertions`, and `time_to_trim_assertions` : Objects describing how long
      each stage of the algorithm took. Fields are:
      * `seconds` : The number of seconds taken at this stage.
      * `work` : An integer indicating the number of steps taken in this stage. For finding winners, it is states in the elimination
        order. For finding assertions, it is the number of elements passing through the priority queue. For trimming, it is the 
        number of nodes of the tree searched (some may be searched twice).

# What if I don't trust it?

Very wise. After all there is no point doing an audit if you can't trust the audit.

The computation RAIRE performs is difficult for a human to repeat, but fortunately
it is much more reasonable for a human to verify that the assertions it suggests 
are indeed sufficient to imply that a particular candidate was the true winner.

See [A guide to Raire](TODO) for details.

# Internal tests

```bash
cargo test
```

# Running original RAIRE examples and interpreting the answers.

There is a program produced, `parse_raire_csv` that reads the original example files in [https://github.com/michelleblom/audit-irv-cp/tree/raire-branch](https://github.com/michelleblom/audit-irv-cp/tree/raire-branch) and
produces a RAIRE JSON format. Run `./target/release/parse_raire_csv --help` for
all options.

There is a program produced, `describe` that takes the JSON output of `raire`
and prints it in a human readable form.

Example:

```bash
./target/release/parse_raire_csv  ../audit-irv-cp/USIRV/SFDA_2019_Nov8Partial.raire
./target/release/raire SFDA_2019_Nov8Partial.json 
./target/release/describe SFDA_2019_Nov8Partial_out.json
```

# Importing from ConcreteSTV or Preflib formats

[ConcreteSTV](https://github.com/AndrewConway/ConcreteSTV) has a format for STV data. IRV data can be considered a subset of STV, 
and _ConcreteSTV_ files can be converted to _raire-rs_ files using [ConcreteSTVToRaire](https://github.com/AndrewConway/ConcreteSTVToRaire).

ConcreteSTV files can be obtained
* By downloading from [vote.andrewconway.org](https://vote.andrewconway.org) 
* Using ConcreteSTV to load files from various electoral commissions
* Using ConcreteSTV to convert [Preflib](https://www.preflib.org/) `.soi` or `.soc` files to ConcreteSTV format.


## Copyright

This program is Copyright 2023 Andrew Conway.
It is based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch

This file is part of raire-rs.

raire-rs is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

raire-rs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with raire-rs.  If not, see <https://www.gnu.org/licenses/>.

## Other copyrights

This repository contains some files derived from data sources with their
own separate copyrights. These files are licensed as above to the extent that they are the work
of contributors to raire-rs, and maintain the original copyright to the appropriate
extent.
* Australian Examples/NSW Local Government/
  These lists are partially derived from data on the
  [NSW Electoral Commission website](https://www.elections.nsw.gov.au), which
  is © State of New South Wales through the NSW Electoral Commission
  and licensed under the [Creative Commons Attribution 4.0 License](https://creativecommons.org/licenses/by/4.0/) (CCA License).
  Thank you to the State of New South Wales for the use of such a license allowing us to use
  this real election data as test data.

This should not be taken as an endorsement of raire-rs by any organisation listed here.

