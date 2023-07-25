# RAIRE-rs

# Important Note: This program is not yet ready for production use.

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
./target/release/raire example_input/a_guide_to_RAIRE_eg.json out.json
```

This will make a file `out.json` in the current directory listing the assertions.

# How to run as a web service

There is a command line program called `raire-webserver` that 
listens at [http://localhost:3000/raire](http://localhost:3000/raire) for a
POST request providing the input JSON and returning the output JSON.

There is a human readable demo also provided at [http://localhost:3000/](http://localhost:3000/)
demonstrating the use of the API and the interpretation of the result.


```bash
./target/release/raire-webserver
```

# JSON input format

TBD

# JSON output format

TBD

# What if I don't trust it?

Very wise. After all there is no point doing an audit if you can't trust the audit.

The computation RAIRE performs is difficult for a human to repeat, but fortunately
it is much more reasonable for a human to verify that the assertions it suggests 
are indeed sufficient to imply that a particular candidate was the true winner.

See [TBD] for details.

# Internal tests

```bash
cargo test
```

Some of these currently fail. See the important note at the top of this file.

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
along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.
