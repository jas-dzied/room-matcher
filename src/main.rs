use anyhow::{anyhow, Result};
use colored::Colorize;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::env;
use std::path::Path;
use std::{collections::HashMap, fs};
use toml::Table;

mod logger;

type Constraints = HashMap<String, (Vec<String>, Vec<String>)>;

const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Debug)]
struct Solution {
    result: Vec<(String, String)>,
    preferred: usize,
    accepted: usize,
    unpreferred: usize,
}

fn solve_constraints(
    people: Vec<String>,
    constraints: &Constraints,
    rng: &mut ThreadRng,
) -> Result<Solution> {
    let mut remaining_people = people;
    remaining_people.shuffle(rng);

    let mut result = vec![];
    let mut num_preferred = 0;
    let mut num_accepted = 0;
    let mut num_unpreferred = 0;

    while !remaining_people.is_empty() {
        let person = remaining_people
            .pop()
            .ok_or_else(|| anyhow!("List of remaining people is empty"))?
            .clone();

        let preferred_people = &constraints
            .get(&person)
            .ok_or_else(|| anyhow!("Person not in constraints"))?
            .0;
        let options = preferred_people
            .iter()
            .filter(|x| remaining_people.contains(x))
            .filter(|x| constraints.get(*x).unwrap().0.contains(&person))
            .cloned()
            .collect::<Vec<_>>();

        let unpreferred_people = &constraints
            .get(&person)
            .ok_or_else(|| anyhow!("Person not in constraints"))?
            .1;
        let secondary_options = remaining_people
            .iter()
            .filter(|x| !unpreferred_people.contains(x))
            .filter(|x| !constraints.get(*x).unwrap().1.contains(&person))
            .cloned()
            .collect::<Vec<_>>();

        if !options.is_empty() {
            let choice = options
                .choose(rng)
                .ok_or_else(|| anyhow!("person not found in options"))?;
            let index = remaining_people
                .iter()
                .position(|x| x == choice)
                .ok_or_else(|| anyhow!("person not found in remaining_people"))?;
            result.push((person, choice.clone()));
            remaining_people.remove(index);
            num_preferred += 1;
        } else if !secondary_options.is_empty() {
            let choice = secondary_options
                .choose(rng)
                .ok_or_else(|| anyhow!("person not found in secondary_options"))?;
            let index = remaining_people
                .iter()
                .position(|x| x == choice)
                .ok_or_else(|| anyhow!("person not found in remaining_people"))?;
            result.push((person, choice.clone()));
            remaining_people.remove(index);
            num_accepted += 1;
        } else {
            let choice = remaining_people
                .choose(rng)
                .ok_or_else(|| anyhow!("person not found in remaining_people"))?;
            let index = remaining_people
                .iter()
                .position(|x| x == choice)
                .ok_or_else(|| anyhow!("person not found in remaining_people"))?;
            result.push((person, choice.clone()));
            remaining_people.remove(index);
            num_unpreferred += 1;
        }
    }

    Ok(Solution {
        result,
        preferred: num_preferred,
        accepted: num_accepted,
        unpreferred: num_unpreferred,
    })
}

fn load_config_file(path: &str) -> Result<(i64, Vec<String>, Constraints)> {
    let log = logger::Logger::info(&format!(
        "{} {}",
        "Loading config file from".truecolor(100, 100, 100),
        Path::new(path).canonicalize()?.display()
    ))?;
    let text = fs::read_to_string(path)?;
    let value = text.parse::<Table>()?;

    let config = value["config"]
        .as_table()
        .ok_or_else(|| anyhow!("Failed to convert to table"))?;
    let num_solutions = config["solutions"]
        .as_integer()
        .ok_or_else(|| anyhow!("Failed to convert to integer"))?;

    let mut people = vec![];
    let mut constraints = HashMap::new();
    log.end();

    let log = logger::Logger::info("Parsing constraints".truecolor(100, 100, 100))?;
    for key in value.keys() {
        if key.as_str() != "config" {
            people.push(key.clone());
            let data = value[key]
                .as_table()
                .ok_or_else(|| anyhow!("Failed to convert to table"))?;
            let preferred = data["preferred"]
                .as_array()
                .ok_or_else(|| anyhow!("Failed to convert to array"))?
                .iter()
                .map(|x| {
                    Ok(x.as_str()
                        .ok_or_else(|| anyhow!("Failed to convert to string"))?
                        .to_string())
                })
                .collect::<Result<Vec<_>>>()?;
            let unpreferred = data["unpreferred"]
                .as_array()
                .ok_or_else(|| anyhow!("Failed to convert to array"))?
                .iter()
                .map(|x| {
                    Ok(x.as_str()
                        .ok_or_else(|| anyhow!("Failed to convert to string"))?
                        .to_string())
                })
                .collect::<Result<Vec<_>>>()?;
            constraints.insert(key.clone(), (preferred, unpreferred));
        }
        //println!("{:#?}", best_solutions);
    }
    log.end();
    Ok((num_solutions, people, constraints))
}

fn find_solutions(
    num_solutions: i64,
    people: &[String],
    constraints: &Constraints,
    rng: &mut ThreadRng,
) -> Result<Vec<Solution>> {
    let log = logger::Logger::info(&format!(
        "{} {} {}",
        "Generating".truecolor(100, 100, 100),
        num_solutions.to_string().truecolor(55, 80, 140),
        "solutions".truecolor(100, 100, 100),
    ))?;
    let mut solutions = vec![];
    for _ in 0..num_solutions {
        solutions.push(solve_constraints(
            people.to_owned(),
            &constraints.clone(),
            rng,
        )?);
    }
    log.end();
    Ok(solutions)
}

fn main() -> Result<()> {
    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());

    let (num_solutions, people, constraints) = load_config_file(&config_path)?;

    let log = logger::Logger::info("Initialising rng".truecolor(100, 100, 100))?;
    let mut rng = rand::thread_rng();
    log.end();

    let solutions = find_solutions(num_solutions, &people, &constraints, &mut rng)?;

    let log = logger::Logger::info("Finding optimal solutions".truecolor(100, 100, 100))?;
    let best_preferred = solutions
        .iter()
        .map(|x| x.preferred)
        .max()
        .ok_or_else(|| anyhow!("No solutions"))?;
    let best_solutions = solutions
        .iter()
        .filter(|x| x.preferred == best_preferred)
        .collect::<Vec<_>>();

    let best_accepted = best_solutions
        .iter()
        .map(|x| x.accepted)
        .max()
        .ok_or_else(|| anyhow!("No solutions"))?;
    let best_solutions = best_solutions
        .iter()
        .filter(|x| x.accepted == best_accepted)
        .collect::<Vec<_>>();
    log.end();

    let log = logger::Logger::info(&format!(
        "{} {} {}",
        "Found".truecolor(100, 100, 100),
        best_solutions.len().to_string().truecolor(55, 80, 140),
        "optimal solutions".truecolor(100, 100, 100),
    ))?;
    log.end();

    let log = logger::Logger::info("Selecting solution".truecolor(100, 100, 100))?;
    let solution = best_solutions
        .choose(&mut rng)
        .ok_or_else(|| anyhow!("No solutions found"))?;
    log.end();

    println!(
        "{} preferred matchups:   {}",
        "RESULT".green(),
        solution.preferred.to_string().blue()
    );
    println!(
        "       accepted matchups:    {}",
        solution.accepted.to_string().blue()
    );
    println!(
        "       unpreferred matchups: {}",
        solution.unpreferred.to_string().blue()
    );
    for (i, room) in solution.result.iter().enumerate() {
        println!(
            "       ROOM {}: {} & {}",
            (i + 1),
            room.0.to_string().blue(),
            room.1.to_string().blue()
        );
    }

    Ok(())
}
