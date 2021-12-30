#[macro_use]
extern crate log;

use std::error::Error;

use clap::{App, Arg};
use console::style;
use image::load_from_memory;
use num_format::{Locale, ToFormattedString};

use pokedex::{PokeMatch, Pokemon, PokemonStatus};
use print::{styled_empty_value, Printer};

mod pokedex;
mod print;

mod join {
    use std::convert::identity;

    pub fn not_empty(value: String) -> bool {
        !value.is_empty()
    }

    pub fn filter_and_map<Filter: Fn(String) -> bool, Map: Fn(String) -> String>(
        values: Vec<&str>,
        separator: &str,
        filter_fn: Filter,
        map_fn: Map,
    ) -> String {
        values
            .into_iter()
            .filter_map(|value| {
                if filter_fn(value.to_string()) {
                    Some(map_fn(value.to_string()))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join(separator)
    }

    pub fn filter<F: Fn(String) -> bool>(
        values: Vec<&str>,
        separator: &str,
        filter_fn: F,
    ) -> String {
        filter_and_map(values, separator, filter_fn, identity)
    }
}

fn optional_empty(value: &str) -> Option<&str> {
    if value.is_empty() {
        return None;
    }
    Some(value)
}

async fn download_image(url: &str) -> Result<image::DynamicImage, Box<dyn Error>> {
    info!("downloading image from \"{}\"", url);

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

struct PokemonPrinter {
    pokemon: Pokemon,
    printer: Printer,
}

impl PokemonPrinter {
    fn pokemon_status(&self) -> Option<String> {
        match &self.pokemon.status {
            PokemonStatus::Normal => None,
            status => Some(format!("{} Pokémon", status.display_name())),
        }
    }

    fn pokemon_types(&self) -> String {
        join::filter_and_map(
            vec![&self.pokemon.type_1, &self.pokemon.type_2],
            " | ",
            join::not_empty,
            |pkmn_type| style(pkmn_type).magenta().to_string(),
        )
    }

    fn pokemon_egg_groups(&self) -> String {
        join::filter(
            vec![&self.pokemon.egg_type_1, &self.pokemon.egg_type_2],
            ", ",
            join::not_empty,
        )
    }

    fn pokemon_genders(&self) -> Option<String> {
        self.pokemon.percentage_male.map(|percentage_male| {
            format!(
                "{}% male, {}% female",
                percentage_male,
                100.0 - percentage_male
            )
        })
    }

    fn pokemon_egg_cycles(&self) -> Option<String> {
        self.pokemon.egg_cycle_stats().map(|stats| {
            let range = format!(
                "({}–{} steps)",
                stats.min_steps.to_formatted_string(&Locale::en),
                stats.max_steps.to_formatted_string(&Locale::en),
            );

            format!("{} {}", stats.cycles, style(range).white().dim())
        })
    }

    async fn print_sprite(&self) {
        let url = self.pokemon.sprite_url();

        match download_image(&url).await {
            Err(err) => self.printer.print_failure(&format!("Image: {}", err)),
            Ok(image) => {
                if let Err(_) = self.printer.print_image(&image, 68) {
                    warn!("image failed to print");
                }
            }
        }
    }

    fn print_header(&self) {
        let PokemonPrinter { pokemon, printer } = self;

        printer.print_center(style(&pokemon.name).yellow());

        if let Some(status) = self.pokemon_status() {
            printer.print_center(style(status).green());
        }

        printer.print_center(format!("Generation {}", pokemon.generation));
    }

    fn print_pokedex_section(&self) {
        let PokemonPrinter { pokemon, printer } = self;

        printer.print_section_heading("Pokédex data");

        printer.print_info("National №", style(pokemon.pokedex_number).yellow());

        printer.print_info("Type", style(self.pokemon_types()).magenta());

        printer.print_info("Species", style(&pokemon.species).cyan());

        printer.print_info(
            "Height",
            match pokemon.height_m {
                Some(val) => style(format!("{} m", val)).cyan(),
                None => styled_empty_value(),
            },
        );

        printer.print_info(
            "Weight",
            match pokemon.weight_kg {
                Some(val) => style(format!("{} kg", val)).cyan(),
                None => styled_empty_value(),
            },
        );

        printer.print_info(
            match pokemon.abilities_number {
                1 => "Ability",
                _ => "Abilities",
            },
            style(&pokemon.ability_1).cyan(),
        );

        if !pokemon.ability_2.is_empty() {
            printer.print_info("", style(&pokemon.ability_2).cyan());
        }

        if !pokemon.ability_hidden.is_empty() {
            printer.print_info(
                "",
                format!(
                    "{} {}",
                    style(&pokemon.ability_hidden).cyan(),
                    style("(hidden ability)").dim()
                ),
            );
        }
    }

    fn print_stats_section(&self) {
        let PokemonPrinter { pokemon, printer } = self;

        printer.print_section_heading("Base Stats");
        printer.print_info("HP", style(pokemon.hp).cyan());
        printer.print_info("Attack", style(pokemon.attack).cyan());
        printer.print_info("Defense", style(pokemon.defense).cyan());
        printer.print_info("Sp. Attack", style(pokemon.sp_attack).cyan());
        printer.print_info("Sp. Defense", style(pokemon.sp_defense).cyan());
        printer.print_info("Speed", style(pokemon.speed).cyan());
        printer.print_info("Total", style(pokemon.total_points).cyan().bold());
    }

    fn print_training_section(&self) {
        let PokemonPrinter { pokemon, printer } = self;

        printer.print_section_heading("Training");

        printer.print_info(
            "Catch Rate",
            match pokemon.catch_rate {
                Some(val) => style(val.to_string()).cyan(),
                None => styled_empty_value(),
            },
        );

        printer.print_info(
            "Base Friendship",
            match pokemon.base_friendship {
                Some(val) => style(val.to_string()).cyan(),
                None => styled_empty_value(),
            },
        );

        printer.print_info(
            "Base Experience",
            match pokemon.base_experience {
                Some(val) => style(val.to_string()).cyan(),
                None => styled_empty_value(),
            },
        );

        printer.print_info(
            "Growth Rate",
            match optional_empty(&pokemon.growth_rate) {
                Some(growth_rate) => style(growth_rate.to_owned()).cyan(),
                None => styled_empty_value(),
            },
        );
    }

    fn print_breeding_section(&self) {
        let PokemonPrinter { printer, .. } = self;

        printer.print_section_heading("Breeding");

        printer.print_info(
            "Egg Groups",
            match optional_empty(&self.pokemon_egg_groups()) {
                Some(egg_groups) => style(egg_groups.to_owned()).cyan(),
                None => styled_empty_value(),
            },
        );

        printer.print_info(
            "Gender",
            match self.pokemon_genders() {
                Some(genders) => style(genders).cyan(),
                None => styled_empty_value(),
            },
        );

        printer.print_info(
            "Egg Cycles",
            match self.pokemon_egg_cycles() {
                Some(egg_cycles) => style(egg_cycles).cyan(),
                None => styled_empty_value(),
            },
        );
    }
}

async fn print_pokemon(pokemon: Pokemon, printer: Printer) {
    let poke_printer = PokemonPrinter { pokemon, printer };

    poke_printer.print_sprite().await;
    println!();
    poke_printer.print_header();
    println!();
    poke_printer.print_pokedex_section();
    println!();
    poke_printer.print_stats_section();
    println!();
    poke_printer.print_training_section();
    println!();
    poke_printer.print_breeding_section();
    println!();
    println!();
}

async fn lookup_pokemon_by_name(query: &str) {
    let printer = Printer { width: 80 };
    let results = pokedex::search_by_name(&query, 5);

    for (i, PokeMatch { pokemon, score }) in results.iter().enumerate() {
        info!(
            "match #{}, {} ({}), similarity: {}, distance: {}",
            i + 1,
            &pokemon.name,
            pokemon.pokedex_number,
            score.similarity,
            score.distance,
        );
    }

    match results.first() {
        None => printer.print_failure("Couldn't find any matches"),
        Some(poke_match) => {
            print_pokemon(poke_match.pokemon.clone(), printer).await;
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .value_name("Searches for a Pokèmon")
                .takes_value(true),
        )
        .get_matches();

    let search_query = matches.value_of("search").unwrap_or("");
    lookup_pokemon_by_name(search_query).await;
}
