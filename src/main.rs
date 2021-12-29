extern crate csv;
extern crate strsim;

use std::error::Error;
use std::fmt::Display;

use clap::{App, Arg};
use console::{pad_str, style};
use image::load_from_memory;
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use serde::Deserialize;
use strsim::{jaro_winkler, levenshtein};
use viuer::{print, Config};

#[derive(Debug, Deserialize)]
enum PokeStatus {
    Normal,
    Legendary,
    Mythical,
    #[serde(rename(deserialize = "Sub Legendary"))]
    SubLegendary,
}

#[derive(Debug, Deserialize)]
struct PokeRecord {
    pokedex_number: u16,
    name: String,
    generation: u8,
    status: PokeStatus,
    species: String,
    type_1: String,
    type_2: String,
    height_m: Option<f32>,
    weight_kg: Option<f32>,
    abilities_number: u8,
    ability_1: String,
    ability_2: String,
    ability_hidden: String,
    total_points: u16,
    hp: u16,
    attack: u16,
    defense: u16,
    sp_attack: u16,
    sp_defense: u16,
    speed: u16,
    catch_rate: Option<u16>,
    base_friendship: Option<u16>,
    base_experience: Option<u16>,
    growth_rate: String,
    egg_type_number: u8,
    egg_type_1: String,
    egg_type_2: String,
    percentage_male: Option<f32>,
    egg_cycles: Option<u16>,
}

#[derive(Debug)]
struct DistanceRecord {
    poke: PokeRecord,
    levenshtein: usize,
    jaro_winkler: f64,
}

impl DistanceRecord {
    pub fn new(poke: PokeRecord, query: &str) -> Self {
        let name = &poke.name.to_lowercase();

        DistanceRecord {
            poke,
            levenshtein: levenshtein(name, query),
            jaro_winkler: jaro_winkler(name, query),
        }
    }
}

pub static POKEDEX_CSV: &[u8] = include_bytes!("../data/pokedex.csv");

async fn dex(query: &str) -> Result<(), Box<dyn Error>> {
    println!("Searching for: {}", query);

    let search_query = query.to_lowercase();
    let mut poke_distances = Vec::new();
    let mut csv_reader = csv::Reader::from_reader(POKEDEX_CSV);

    for result in csv_reader.deserialize() {
        let poke_record: PokeRecord = result?;
        let distance_record = DistanceRecord::new(poke_record, &search_query);
        poke_distances.push(distance_record);
    }

    poke_distances.sort_by(|a, b| {
        b.jaro_winkler
            .partial_cmp(&a.jaro_winkler)
            .unwrap()
            .then(a.levenshtein.cmp(&b.levenshtein))
    });

    // poke_distances.truncate(20);
    // for poke_distance in poke_distances {
    //     println!("{:?}", poke_distance);
    // }

    let pokemon = poke_distances.first();
    match pokemon {
        None => print_failure("Couldn't find any matches"),
        Some(p) => {
            println!();
            println!();

            print_poke_image(&p.poke).await?;

            println!();

            println!("{}", style(center(&p.poke.name)).yellow());

            match &p.poke.status {
                PokeStatus::Normal => {} // do nothing
                status => println!(
                    "{}",
                    style(center(match status {
                        PokeStatus::Legendary | PokeStatus::SubLegendary => "Legendary Pokémon",
                        PokeStatus::Mythical => "Mythical Pokémon",
                        PokeStatus::Normal => "",
                    }))
                    .green(),
                ),
            }

            println!("{}", center(&format!("Generation {}", p.poke.generation)));

            println!();

            print_section_heading("Pokédex data");
            print_info("National №", style(p.poke.pokedex_number).yellow());
            print_info(
                "Type",
                style(join(vec![&p.poke.type_1, &p.poke.type_2], " | ")).magenta(),
            );
            print_info("Species", style(&p.poke.species).cyan());
            print_info(
                "Height",
                match p.poke.height_m {
                    Some(val) => style(format!("{} m", val)).cyan(),
                    None => style(String::from("-")).dim(),
                },
            );
            print_info(
                "Weight",
                match p.poke.weight_kg {
                    Some(val) => style(format!("{} kg", val)).cyan(),
                    None => style(String::from("-")).dim(),
                },
            );

            let abilities_label = match p.poke.abilities_number {
                1 => "Ability",
                _ => "Abilities",
            };
            print_info(abilities_label, style(&p.poke.ability_1).cyan());
            if !p.poke.ability_2.is_empty() {
                print_info("", style(&p.poke.ability_2).cyan());
            }
            if !p.poke.ability_hidden.is_empty() {
                print_info(
                    "",
                    format!(
                        "{} {}",
                        style(&p.poke.ability_hidden).cyan(),
                        style("(hidden ability)").dim()
                    ),
                );
            }

            println!();

            print_section_heading("Base Stats");
            print_info("HP", style(p.poke.hp).cyan());
            print_info("Attack", style(p.poke.attack).cyan());
            print_info("Defense", style(p.poke.defense).cyan());
            print_info("Sp. Attack", style(p.poke.sp_attack).cyan());
            print_info("Sp. Defense", style(p.poke.sp_defense).cyan());
            print_info("Speed", style(p.poke.speed).cyan());
            print_info("Total", style(p.poke.total_points).cyan().bold());

            println!();

            print_section_heading("Training");
            print_info(
                "Catch Rate",
                match p.poke.catch_rate {
                    Some(val) => style(val.to_string()).cyan(),
                    None => style(String::from("-")).dim(),
                },
            );
            print_info(
                "Base Friendship",
                match p.poke.base_friendship {
                    Some(val) => style(val.to_string()).cyan(),
                    None => style(String::from("-")).dim(),
                },
            );
            print_info(
                "Base Experience",
                match p.poke.base_experience {
                    Some(val) => style(val.to_string()).cyan(),
                    None => style(String::from("-")).dim(),
                },
            );
            print_info("Growth Rate", style(&p.poke.growth_rate).cyan());

            println!();

            if p.poke.egg_type_number > 0 {
                print_section_heading("Breeding");
                print_info(
                    "Egg Groups",
                    style(join(vec![&p.poke.egg_type_1, &p.poke.egg_type_2], ", ")).cyan(),
                );
                print_info(
                    "Gender",
                    match p.poke.percentage_male {
                        Some(val) => {
                            style(format!("{}% male, {}% female", val, 100.0 - val)).cyan()
                        }
                        None => style(String::from("-")).dim(),
                    },
                );
                print_info(
                    "Egg Cycles",
                    match p.poke.egg_cycles {
                        Some(val) => {
                            let egg_cycle_factor = 257;
                            let max = val * egg_cycle_factor;
                            let min = ((val - 1) * egg_cycle_factor) + 1;

                            style(format!(
                                "{} {}",
                                val,
                                style(format!(
                                    "({}–{} steps)",
                                    min.to_formatted_string(&Locale::en),
                                    max.to_formatted_string(&Locale::en),
                                ))
                                .white()
                                .dim(),
                            ))
                            .cyan()
                        }
                        None => style(String::from("-")).dim(),
                    },
                );
            }

            println!();
            println!();
        }
    }

    Ok(())
}

fn join(list: Vec<&str>, separator: &str) -> String {
    list.into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>()
        .join(separator)
}

fn print_info<T1: Display, T2: Display>(label: T1, info: T2) {
    println!("{:>39}  {}", style(label), info);
}

fn print_section_heading(heading: &str) {
    print_info(style(heading).bold(), "");
}

fn center(message: &str) -> String {
    pad_str(message, 80, console::Alignment::Center, None).to_string()
}

fn print_failure(message: &str) {
    println!();
    println!(
        "{}",
        style(pad_str(message, 80, console::Alignment::Center, None)).red()
    );
    println!();
}

fn slugify_poke_name(name: &str) -> String {
    let mega_re = Regex::new("^mega-(?P<name>.+?)(?P<xy>-x|-y)?$").unwrap();
    let n = name.to_lowercase().replace(" ", "-");
    mega_re.replace(&n, "$name-mega$xy").to_string()
}

async fn print_poke_image(poke: &PokeRecord) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/itsjavi/pokemon-assets/master/assets/img/pokemon/{}.png",
        slugify_poke_name(&poke.name)
    );

    let image_result = image_from_url(&url).await;

    match image_result {
        Err(err) => print_failure(&format!("Image: {}", err)),
        Ok(image) => {
            let conf = Config {
                transparent: true,
                absolute_offset: false,
                x: 6,
                y: 0,
                width: Some(68),
                ..Default::default()
            };
            print(&image, &conf).expect("Image printing failed.");
        }
    }

    Ok(())
}

async fn image_from_url(url: &str) -> Result<image::DynamicImage, Box<dyn Error>> {
    let res = reqwest::get(url).await?;

    match res.status() {
        status if status.is_success() => {
            let bytes = res.bytes().await?;
            let image = load_from_memory(&bytes).unwrap();
            Ok(image)
        }
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let matches = App::new("PokeRust")
        .arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .value_name("Searches for a Pokemon")
                .takes_value(true),
        )
        .get_matches();

    let search = matches.value_of("search").unwrap_or("");
    if let Err(err) = dex(search).await {
        println!("error during search: {}", err);
        std::process::exit(1);
    }

    Ok(())
}
