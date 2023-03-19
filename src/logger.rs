use anyhow::Result;
use colored::Colorize;
use std::{
    fmt,
    io::{self, Write},
    time::{Duration, Instant},
};

pub struct Logger {
    start: Instant,
}

pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
}

impl TimeUnit {
    pub fn next(&self) -> TimeUnit {
        match self {
            TimeUnit::Nanoseconds => TimeUnit::Microseconds,
            TimeUnit::Microseconds => TimeUnit::Milliseconds,
            TimeUnit::Milliseconds => TimeUnit::Seconds,
            _ => unreachable!(),
        }
    }
    pub fn repr(&self) -> &str {
        match self {
            TimeUnit::Nanoseconds => "ns",
            TimeUnit::Microseconds => "μs",
            TimeUnit::Milliseconds => "ms",
            TimeUnit::Seconds => "s",
        }
    }
}

fn display_duration(duration: Duration) -> (u128, TimeUnit) {
    let mut unit = TimeUnit::Nanoseconds;
    let mut time_since_start = duration.as_nanos();
    if time_since_start > 5000 {
        time_since_start /= 1000;
        unit = unit.next();
    }
    if time_since_start > 5000 {
        time_since_start /= 1000;
        unit = unit.next();
    }
    if time_since_start > 5000 {
        time_since_start /= 1000;
        unit = unit.next();
    }
    (time_since_start, unit)
}

impl Logger {
    pub fn info<T: fmt::Display>(text: T) -> Result<Logger> {
        print!("{} {}", " INFO ".yellow(), text,);
        io::stdout().flush()?;
        Ok(Logger {
            start: Instant::now(),
        })
    }
    pub fn end(self) {
        let elapsed = self.start.elapsed();
        let (elapsed, unit) = display_duration(elapsed);
        println!(
            " {} {}{}",
            "took".truecolor(150, 150, 150),
            elapsed.to_string().truecolor(150, 150, 150),
            unit.repr().truecolor(150, 150, 150)
        );
    }
}
