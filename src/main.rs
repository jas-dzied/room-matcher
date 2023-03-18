use colored::Colorize;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::env;
use std::path::Path;
use std::time::Instant;
use std::{collections::HashMap, fs};
use toml::Table;

const DEFAULT_CONFIG_PATH: &str = "config.toml";

enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
}

#[derive(Debug)]
struct Solution {
    result: Vec<(String, String)>,
    preferred: usize,
    accepted: usize,
    unpreferred: usize,
}

impl TimeUnit {
    fn next(&self) -> TimeUnit {
        match self {
            TimeUnit::Nanoseconds => TimeUnit::Microseconds,
            TimeUnit::Microseconds => TimeUnit::Milliseconds,
            TimeUnit::Milliseconds => TimeUnit::Seconds,
            _ => panic!("bad"),
        }
    }
    fn repr(&self) -> &str {
        match self {
            TimeUnit::Nanoseconds => "ns",
            TimeUnit::Microseconds => "μs",
            TimeUnit::Milliseconds => "ms",
            TimeUnit::Seconds => "s",
        }
    }
}

fn info<T: std::fmt::Display>(message: T, start: Instant) {
    let mut time_since_start = start.elapsed().as_nanos();
    let mut unit = TimeUnit::Nanoseconds;
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
    println!(
        "{} {} {}{}",
        " INFO ".yellow(),
        message,
        time_since_start.to_string().truecolor(150, 150, 150),
        unit.repr().truecolor(150, 150, 150)
    )
}

fn error<T: std::fmt::Display>(message: T, start: Instant) {
    let mut time_since_start = start.elapsed().as_nanos();
    let mut unit = TimeUnit::Nanoseconds;
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
    println!(
        "{} {} {}{}",
        " ERROR".red(),
        message,
        time_since_start.to_string().truecolor(150, 150, 150),
        unit.repr().truecolor(150, 150, 150)
    )
}

fn solve_constraints(
    people: Vec<String>,
    constraints: HashMap<String, (Vec<String>, Vec<String>)>,
    rng: &mut ThreadRng,
    start: Instant,
) -> Solution {
    let mut remaining_people = people.clone();
    remaining_people.shuffle(rng);

    let mut result = vec![];
    let mut num_preferred = 0;
    let mut num_accepted = 0;
    let mut num_unpreferred = 0;

    while !remaining_people.is_empty() {
        let person = remaining_people.pop().unwrap().clone();

        let preferred_people = &constraints.get(&person).unwrap().0;
        let options = preferred_people
            .iter()
            .filter(|x| remaining_people.contains(x))
            .filter(|x| constraints.get(*x).unwrap().0.contains(&person))
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        let unpreferred_people = &constraints.get(&person).unwrap().1;
        let secondary_options = remaining_people
            .iter()
            .filter(|x| !unpreferred_people.contains(x))
            .filter(|x| !constraints.get(*x).unwrap().1.contains(&person))
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        if !options.is_empty() {
            let choice = options.choose(rng).unwrap();
            let index = remaining_people.iter().position(|x| x == choice).unwrap();
            result.push((person, choice.clone()));
            remaining_people.remove(index);
            num_preferred += 1;
        } else if !secondary_options.is_empty() {
            let choice = secondary_options.choose(rng).unwrap();
            let index = remaining_people.iter().position(|x| x == choice).unwrap();
            result.push((person, choice.clone()));
            remaining_people.remove(index);
            num_accepted += 1;
        } else {
            if remaining_people.is_empty() {
                error(
                    "Not enough people to fill rooms. Maybe you forgot to add a person?",
                    start,
                );
                panic!("Exiting program");
            }
            let choice = remaining_people.choose(rng).unwrap();
            let index = remaining_people.iter().position(|x| x == choice).unwrap();
            result.push((person, choice.clone()));
            remaining_people.remove(index);
            num_unpreferred += 1;
        }
    }

    Solution {
        result,
        preferred: num_preferred,
        accepted: num_accepted,
        unpreferred: num_unpreferred,
    }
}

fn main() {
    let start = Instant::now();

    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());

    let config_full_path = Path::new(&config_path);

    info(
        &format!(
            "{} {}",
            "Loading config file from".truecolor(100, 100, 100),
            config_full_path.canonicalize().unwrap().display()
        ),
        start,
    );
    let text = fs::read_to_string(config_path).unwrap_or_else(|_| {
        error("Failed to find config file", start);
        panic!("Exiting program");
    });
    let value = text.parse::<Table>().unwrap();

    let config = value["config"].as_table().unwrap();
    let num_solutions = config["solutions"].as_integer().unwrap();

    let mut people = vec![];
    let mut constraints = HashMap::new();

    info("Parsing constraints".truecolor(100, 100, 100), start);
    for key in value.keys() {
        if key.as_str() != "config" {
            people.push(key.clone());
            let data = value[key].as_table().unwrap();
            let preferred = data["preferred"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            let unpreferred = data["unpreferred"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            constraints.insert(key.clone(), (preferred, unpreferred));
        }
    }

    info("Initialising rng".truecolor(100, 100, 100), start);
    let mut rng = rand::thread_rng();
    info(
        &format!(
            "{} {} {}",
            "Generating".truecolor(100, 100, 100),
            num_solutions.to_string().truecolor(55, 80, 140),
            "solutions".truecolor(100, 100, 100),
        ),
        start,
    );
    let solutions = (0..num_solutions)
        .map(|_| solve_constraints(people.clone(), constraints.clone(), &mut rng, start))
        .collect::<Vec<_>>();

    let best_preferred = solutions.iter().map(|x| x.preferred).max().unwrap();
    let best_solutions = solutions
        .iter()
        .filter(|x| x.preferred == best_preferred)
        .collect::<Vec<_>>();

    let best_accepted = best_solutions.iter().map(|x| x.accepted).max().unwrap();
    let best_solutions = best_solutions
        .iter()
        .filter(|x| x.accepted == best_accepted)
        .collect::<Vec<_>>();

    info(
        &format!(
            "{} {} {}",
            "Found".truecolor(100, 100, 100),
            best_solutions.len().to_string().truecolor(55, 80, 140),
            "optimal solutions".truecolor(100, 100, 100),
        ),
        start,
    );

    info("Selecting solution".truecolor(100, 100, 100), start);
    let solution = best_solutions.choose(&mut rng).unwrap();
    println!(
        "{} {}   {}",
        "RESULT".green(),
        "preferred matchups:",
        solution.preferred.to_string().blue()
    );
    println!(
        "       {}    {}",
        "accepted matchups:",
        solution.accepted.to_string().blue()
    );
    println!(
        "       {} {}",
        "unpreferred matchups:",
        solution.unpreferred.to_string().blue()
    );
    for (i, room) in solution.result.iter().enumerate() {
        println!(
            "       ROOM {}: {} & {}",
            (i + 1).to_string(),
            room.0.to_string().blue(),
            room.1.to_string().blue()
        )
    }

    //println!("{:#?}", best_solutions);
}
