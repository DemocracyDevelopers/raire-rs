// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


use std::ops::Sub;
use std::time::{Duration, Instant};
use serde::Deserialize;
use serde::Serialize;

/// A check to see that we are not taking too long.
/// Allows efficient checking against clock time taken or work done.
pub struct TimeOut {
    start_time : Instant,
    work_done : u64,
    work_limit : Option<u64>,
    duration_limit : Option<Duration>,
}

impl TimeOut {
    /// Make a new timeout structure.
    pub fn new(work_limit : Option<u64>,duration_limit : Option<Duration>) -> Self {
        let start_time = Instant::now();
        TimeOut{start_time,work_done:0,work_limit,duration_limit}
    }

    /// make a dummy timer that will never timeout
    pub fn never() -> Self { Self::new(None,None) }

    pub fn clock_time_taken_since_start(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn get_work_done(&self) -> u64 { self.work_done }

    pub fn time_taken(&self) -> TimeTaken {
        TimeTaken{ work: self.work_done, seconds: self.clock_time_taken_since_start().as_secs_f64() }
    }

    /// increments work_done by 1, and returns true if a limit is exceeded
    /// * only checks duration every 100 calls.
    pub fn quick_check_timeout(&mut self) -> bool {
        self.work_done+=1;
        if let Some(work_limit) = self.work_limit {
            if self.work_done>work_limit { return true; }
        }
        if let Some(duration_limit) = self.duration_limit {
            if self.clock_time_taken_since_start()>duration_limit { return true; }
        }
        false
    }
}

#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
/// A measure of the time taken to do something.
pub struct TimeTaken {
    work : u64,
    seconds : f64,
}

impl Sub for TimeTaken {
    type Output = TimeTaken;

    fn sub(self, rhs: Self) -> Self::Output {
        TimeTaken{work:self.work-rhs.work,seconds:self.seconds-rhs.seconds}
    }
}


