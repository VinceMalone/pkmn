use std::cmp::Ordering;

use regex::Regex;
use serde::Deserialize;

pub struct EggCycleStats {
    pub cycles: u16,
    pub max_steps: u16,
    pub min_steps: u16,
}

impl EggCycleStats {
    fn new(cycles: u16) -> Self {
        const EGG_CYCLE_FACTOR: u16 = 257;

        EggCycleStats {
            cycles,
            max_steps: cycles * EGG_CYCLE_FACTOR,
            min_steps: ((cycles - 1) * EGG_CYCLE_FACTOR) + 1,
        }
    }
}

#[derive(Clone, Deserialize)]
pub enum PokemonStatus {
    Normal,
    Legendary,
    Mythical,
    #[serde(rename(deserialize = "Sub Legendary"))]
    SubLegendary,
}

impl PokemonStatus {
    pub fn display_name(&self) -> String {
        match self {
            PokemonStatus::Legendary | PokemonStatus::SubLegendary => "Legendary",
            PokemonStatus::Mythical => "Mythical",
            PokemonStatus::Normal => "Normal",
        }
        .to_string()
    }
}

#[derive(Clone, Deserialize)]
pub struct Pokemon {
    pub pokedex_number: u16,
    pub name: String,
    pub generation: u8,
    pub status: PokemonStatus,
    pub species: String,
    pub type_1: String,
    pub type_2: String,
    pub height_m: Option<f32>,
    pub weight_kg: Option<f32>,
    pub abilities_number: u8,
    pub ability_1: String,
    pub ability_2: String,
    pub ability_hidden: String,
    pub total_points: u16,
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
    pub catch_rate: Option<u16>,
    pub base_friendship: Option<u16>,
    pub base_experience: Option<u16>,
    pub growth_rate: String,
    pub egg_type_number: u8,
    pub egg_type_1: String,
    pub egg_type_2: String,
    pub percentage_male: Option<f32>,
    pub egg_cycles: Option<u16>,
}

impl Pokemon {
    pub fn egg_cycle_stats(&self) -> Option<EggCycleStats> {
        self.egg_cycles.map(|cycles| EggCycleStats::new(cycles))
    }

    fn sprite_name_slug(&self) -> String {
        let mega_re = Regex::new("^mega-(?P<name>.+?)(?P<xy>-x|-y)?$").unwrap();
        let n = self
            .name
            .to_lowercase()
            .replace(" ", "-")
            .replace(".", "")
            .replace(":", "")
            .replace("'", "")
            .replace("é", "e") // TODO: do this for all types
            .replace("♀", "-f")
            .replace("♂", "-m");
        mega_re.replace(&n, "$name-mega$xy").to_string()
    }

    pub fn sprite_url(&self) -> String {
        format!(
            "https://raw.githubusercontent.com/itsjavi/pokemon-assets/master/assets/img/pokemon/{}.png",
            self.sprite_name_slug()
        )
    }
}

pub struct MatchScore {
    pub distance: usize,
    pub similarity: f64,
}

impl MatchScore {
    pub fn new(value: &str, query: &str) -> Self {
        MatchScore {
            distance: strsim::levenshtein(value, query),
            similarity: strsim::jaro_winkler(value, query),
        }
    }

    fn compare(a: &MatchScore, b: &MatchScore) -> Ordering {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap()
            .then(a.distance.cmp(&b.distance))
    }
}

pub struct PokeMatch {
    pub pokemon: Pokemon,
    pub score: MatchScore,
}

static POKEDEX_CSV: &[u8] = include_bytes!("../data/pokedex.csv");

pub fn search_by_name(query: &str, limit: usize) -> Vec<PokeMatch> {
    let search_query = query.to_lowercase();
    let mut results = Vec::new();
    let mut csv_reader = csv::Reader::from_reader(POKEDEX_CSV);

    for result in csv_reader.deserialize() {
        let pokemon: Pokemon = result.unwrap();
        let match_score = MatchScore::new(&pokemon.name.to_lowercase(), &search_query);
        results.push(PokeMatch {
            pokemon,
            score: match_score,
        });
    }

    results.sort_by(|a, b| MatchScore::compare(&a.score, &b.score));
    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_match() {
        let results = search_by_name("x", 1);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn multiple_matches() {
        let results = search_by_name("x", 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn exact_match() {
        let result = &search_by_name("charizard", 1)[0];
        assert_eq!(result.pokemon.name, "Charizard");
        assert_eq!(result.score.similarity, 1.0);
        assert_eq!(result.score.distance, 0);
    }

    #[test]
    fn close_match() {
        let result = &search_by_name("charzad", 1)[0];
        assert_eq!(result.pokemon.name, "Charizard");
        assert_eq!(result.score.similarity, 0.9555555555555555);
        assert_eq!(result.score.distance, 2);
    }

    #[test]
    fn loose_match() {
        let result = &search_by_name("char", 1)[0];
        assert_eq!(result.pokemon.name, "Charizard");
        assert_eq!(result.score.similarity, 0.888888888888889);
        assert_eq!(result.score.distance, 5);
    }

    impl Pokemon {
        fn default() -> Self {
            Pokemon {
                pokedex_number: 0,
                name: "".to_string(),
                generation: 0,
                status: PokemonStatus::Normal,
                species: "".to_string(),
                type_1: "".to_string(),
                type_2: "".to_string(),
                height_m: None,
                weight_kg: None,
                abilities_number: 0,
                ability_1: "".to_string(),
                ability_2: "".to_string(),
                ability_hidden: "".to_string(),
                total_points: 0,
                hp: 0,
                attack: 0,
                defense: 0,
                sp_attack: 0,
                sp_defense: 0,
                speed: 0,
                catch_rate: None,
                base_friendship: None,
                base_experience: None,
                growth_rate: "".to_string(),
                egg_type_number: 0,
                egg_type_1: "".to_string(),
                egg_type_2: "".to_string(),
                percentage_male: None,
                egg_cycles: None,
            }
        }
    }

    #[test]
    fn sprite_name_slug() {
        let pkmn = Pokemon {
            name: String::from("Magneton"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "magneton");
    }

    #[test]
    fn sprite_name_slug_dot() {
        let pkmn = Pokemon {
            name: String::from("Mr. Mime"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "mr-mime");
    }

    #[test]
    fn sprite_name_slug_colon() {
        let pkmn = Pokemon {
            name: String::from("Type: Null"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "type-null");
    }

    #[test]
    fn sprite_name_slug_apos() {
        let pkmn = Pokemon {
            name: String::from("Farfetch'd"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "farfetchd");
    }

    #[test]
    fn sprite_name_slug_accent() {
        let pkmn = Pokemon {
            name: String::from("Flabébé"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "flabebe");
    }

    #[test]
    fn sprite_name_slug_female() {
        let pkmn = Pokemon {
            name: String::from("Nidoran♀"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "nidoran-f");
    }

    #[test]
    fn sprite_name_slug_male() {
        let pkmn = Pokemon {
            name: String::from("Nidoran♂"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "nidoran-m");
    }

    #[test]
    fn sprite_name_slug_mega() {
        let pkmn = Pokemon {
            name: String::from("Mega Steelix"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "steelix-mega");
    }

    #[test]
    fn sprite_name_slug_mega_x() {
        let pkmn = Pokemon {
            name: String::from("Mega Charizard X"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "charizard-mega-x");
    }

    #[test]
    fn sprite_name_slug_mega_y() {
        let pkmn = Pokemon {
            name: String::from("Mega Mewtwo Y"),
            ..Pokemon::default()
        };
        assert_eq!(pkmn.sprite_name_slug(), "mewtwo-mega-y");
    }

    #[test]
    fn egg_cycle_stats() {
        let stats = EggCycleStats::new(17);
        assert_eq!(stats.cycles, 17);
        assert_eq!(stats.max_steps, 4369);
        assert_eq!(stats.min_steps, 4113);
    }

    #[test]
    fn match_compare_equal() {
        let a = MatchScore {
            distance: 1,
            similarity: 1.0,
        };
        let b = MatchScore {
            distance: 1,
            similarity: 1.0,
        };
        assert_eq!(MatchScore::compare(&a, &b), Ordering::Equal);
    }

    #[test]
    fn match_compare_similarity_is_highest_priority() {
        let a = MatchScore {
            distance: 1,
            similarity: 0.75,
        };
        let b = MatchScore {
            distance: 2,
            similarity: 0.25,
        };
        assert_eq!(MatchScore::compare(&a, &b), Ordering::Less);
        assert_eq!(MatchScore::compare(&b, &a), Ordering::Greater);
    }

    #[test]
    fn match_compare_distance_is_tie_breaker() {
        let a = MatchScore {
            distance: 2,
            similarity: 0.5,
        };
        let b = MatchScore {
            distance: 1,
            similarity: 0.5,
        };
        assert_eq!(MatchScore::compare(&b, &a), Ordering::Less);
        assert_eq!(MatchScore::compare(&a, &b), Ordering::Greater);
    }
}
