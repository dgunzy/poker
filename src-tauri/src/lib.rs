#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod storage;
mod store_backend;

use std::sync::Arc;

use poker_core::{Card, Rank, Suit};
use tauri::Manager;
use storage::{Preferences, StorageState, TrainingMode};
use store_backend::StoreBackend;
use tauri_plugin_store::StoreExt;
use poker_eval::GameVariant;
use poker_sim::{Simulator, SimulationInput};
use poker_sim::training::{self, GameTrainingInfo};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct TrainingConfig {
    percentile_guessing: PercentileGuessingConfig,
}

#[derive(Debug, Deserialize)]
struct PercentileGuessingConfig {
    exact_threshold: f64,
    close_threshold: f64,
    default_draw_count: u32,
}

#[derive(Debug, Deserialize)]
struct UiConfig {
    suits: std::collections::HashMap<String, String>,
    feedback: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct GamesConfig {
    games: std::collections::HashMap<String, GameDef>,
}

#[derive(Debug, Deserialize)]
struct GameDef {
    name: String,
    hand_size: u8,
    deck_size: u8,
    #[serde(default)]
    #[allow(dead_code)] // for future UI hints; see GAME-RULES.md
    simulation_type: Option<String>,
    description: String,
}

#[derive(Debug, Serialize)]
struct GameInfo {
    id: String,
    name: String,
    hand_size: u8,
    deck_size: u8,
    description: String,
}

/// IPC-friendly card representation (rank/suit strings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInput {
    pub rank: String,
    pub suit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationInputIpc {
    pub held_cards: Vec<CardInput>,
    pub dead_cards: Vec<CardInput>,
    pub draw_count: u32,
    pub seed: Option<u64>,
    #[serde(default)]
    pub game_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandResultIpc {
    pub hand: Vec<CardInput>,
    pub rank: u32,
    pub percentile: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResultIpc {
    pub results: Vec<HandResultIpc>,
    pub draw_count: u32,
    pub best_rank: u32,
    pub worst_rank: u32,
    pub seed_used: u64,
}

fn parse_rank(s: &str) -> Option<Rank> {
    match s.to_lowercase().as_str() {
        "two" | "2" => Some(Rank::Two),
        "three" | "3" => Some(Rank::Three),
        "four" | "4" => Some(Rank::Four),
        "five" | "5" => Some(Rank::Five),
        "six" | "6" => Some(Rank::Six),
        "seven" | "7" => Some(Rank::Seven),
        "eight" | "8" => Some(Rank::Eight),
        "nine" | "9" => Some(Rank::Nine),
        "ten" | "t" | "10" => Some(Rank::Ten),
        "jack" | "j" => Some(Rank::Jack),
        "queen" | "q" => Some(Rank::Queen),
        "king" | "k" => Some(Rank::King),
        "ace" | "a" => Some(Rank::Ace),
        _ => None,
    }
}

fn parse_suit(s: &str) -> Option<Suit> {
    match s.to_lowercase().as_str() {
        "clubs" | "c" => Some(Suit::Clubs),
        "diamonds" | "d" => Some(Suit::Diamonds),
        "hearts" | "h" => Some(Suit::Hearts),
        "spades" | "s" => Some(Suit::Spades),
        _ => None,
    }
}

fn card_input_to_card(c: &CardInput) -> Result<Card, String> {
    let rank = parse_rank(&c.rank).ok_or_else(|| format!("Invalid rank: {}", c.rank))?;
    let suit = parse_suit(&c.suit).ok_or_else(|| format!("Invalid suit: {}", c.suit))?;
    Ok(Card::new(rank, suit))
}

fn card_to_input(c: Card) -> CardInput {
    let rank = match c.rank() {
        Rank::Two => "two",
        Rank::Three => "three",
        Rank::Four => "four",
        Rank::Five => "five",
        Rank::Six => "six",
        Rank::Seven => "seven",
        Rank::Eight => "eight",
        Rank::Nine => "nine",
        Rank::Ten => "ten",
        Rank::Jack => "jack",
        Rank::Queen => "queen",
        Rank::King => "king",
        Rank::Ace => "ace",
    };
    let suit = match c.suit() {
        Suit::Clubs => "clubs",
        Suit::Diamonds => "diamonds",
        Suit::Hearts => "hearts",
        Suit::Spades => "spades",
    };
    CardInput {
        rank: rank.to_string(),
        suit: suit.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesIpc {
    pub game_id: String,
    pub draw_count: u32,
    pub training_mode: String,
    pub training_draw_count: u32,
}

fn preferences_to_ipc(p: &Preferences) -> PreferencesIpc {
    PreferencesIpc {
        game_id: p.game_id.clone(),
        draw_count: p.draw_count,
        training_mode: match p.training_mode {
            TrainingMode::Percentile => "percentile",
            TrainingMode::Generator => "generator",
        }
        .to_string(),
        training_draw_count: p.training_draw_count,
    }
}

fn preferences_from_ipc(p: PreferencesIpc) -> Preferences {
    Preferences {
        game_id: p.game_id,
        draw_count: p.draw_count,
        training_mode: match p.training_mode.as_str() {
            "generator" => TrainingMode::Generator,
            _ => TrainingMode::Percentile,
        },
        training_draw_count: p.training_draw_count,
    }
}

#[derive(Debug, Serialize)]
struct TrainingConfigResponse {
    exact_threshold: f64,
    close_threshold: f64,
    default_draw_count: u32,
}

#[derive(Debug, Serialize)]
struct UiConfigResponse {
    suits: std::collections::HashMap<String, String>,
    feedback: std::collections::HashMap<String, String>,
}

#[tauri::command]
fn get_games() -> Result<Vec<GameInfo>, String> {
    let config_toml = include_str!("../../config/games.toml");
    let config: GamesConfig = toml::from_str(config_toml).map_err(|e| e.to_string())?;
    Ok(config
        .games
        .into_iter()
        .map(|(id, g)| GameInfo {
            id: id.to_string(),
            name: g.name,
            hand_size: g.hand_size,
            deck_size: g.deck_size,
            description: g.description,
        })
        .collect())
}

#[tauri::command]
fn get_training_config() -> Result<TrainingConfigResponse, String> {
    let config_toml = include_str!("../../config/training.toml");
    let config: TrainingConfig = toml::from_str(config_toml).map_err(|e| e.to_string())?;
    Ok(TrainingConfigResponse {
        exact_threshold: config.percentile_guessing.exact_threshold,
        close_threshold: config.percentile_guessing.close_threshold,
        default_draw_count: config.percentile_guessing.default_draw_count,
    })
}

#[tauri::command]
fn get_preferences(storage: tauri::State<StorageState>) -> Result<PreferencesIpc, String> {
    storage
        .0
        .load()
        .map(|p| preferences_to_ipc(&p))
        .map_err(|e: storage::StorageError| e.to_string())
}

#[tauri::command]
fn set_preferences(
    storage: tauri::State<StorageState>,
    prefs: PreferencesIpc,
) -> Result<(), String> {
    storage
        .0
        .save(&preferences_from_ipc(prefs))
        .map_err(|e: storage::StorageError| e.to_string())
}

#[tauri::command]
fn get_ui_config() -> Result<UiConfigResponse, String> {
    let config_toml = include_str!("../../config/ui.toml");
    let config: UiConfig = toml::from_str(config_toml).map_err(|e| e.to_string())?;
    Ok(UiConfigResponse {
        suits: config.suits,
        feedback: config.feedback,
    })
}

#[tauri::command]
fn run_simulation(input: SimulationInputIpc) -> Result<SimulationResultIpc, String> {
    let held_cards: Vec<Card> = input
        .held_cards
        .iter()
        .map(card_input_to_card)
        .collect::<Result<Vec<_>, _>>()?;
    let dead_cards: Vec<Card> = input
        .dead_cards
        .iter()
        .map(card_input_to_card)
        .collect::<Result<Vec<_>, _>>()?;

    let game_variant = input
        .game_id
        .as_deref()
        .map(GameVariant::from_game_id)
        .unwrap_or_default();

    let sim_input = SimulationInput {
        held_cards,
        dead_cards,
        draw_count: input.draw_count,
        seed: input.seed,
        game_variant,
    };

    let sim = Simulator::default();
    let result = sim.run(&sim_input).map_err(|e| e.to_string())?;

    Ok(SimulationResultIpc {
        results: result
            .results
            .into_iter()
            .map(|r| HandResultIpc {
                hand: r.hand.iter().copied().map(card_to_input).collect(),
                rank: r.rank,
                percentile: r.percentile,
            })
            .collect(),
        draw_count: result.draw_count,
        best_rank: result.best_rank,
        worst_rank: result.worst_rank,
        seed_used: result.seed_used,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingScenarioInputIpc {
    pub held_cards: Vec<CardInput>,
    pub dead_cards: Vec<CardInput>,
    pub sim_count: u32,
    pub game_id: String,
    pub seed: Option<u64>,
}

/// IPC-friendly TrainingScenario. Frontend expects {rank, suit} for cards, not u8.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrainingScenarioIpc {
    Draw {
        hand: Vec<CardInput>,
        rank: u32,
        percentile: f64,
    },
    Board {
        hole_cards: Vec<CardInput>,
        board: Vec<CardInput>,
        best_hand: Vec<CardInput>,
        rank: u32,
        percentile: f64,
    },
    SplitPot {
        hole_cards: Vec<CardInput>,
        board: Vec<CardInput>,
        best_high_hand: Vec<CardInput>,
        high_rank: u32,
        high_percentile: f64,
        low_qualifies: bool,
        best_low_hand: Option<Vec<CardInput>>,
        low_rank: Option<u32>,
        low_percentile: Option<f64>,
    },
}

fn cards_to_ipc(cards: impl IntoIterator<Item = Card>) -> Vec<CardInput> {
    cards.into_iter().map(card_to_input).collect()
}

fn training_scenario_to_ipc(s: poker_sim::training::TrainingScenario) -> TrainingScenarioIpc {
    use poker_sim::training::TrainingScenario as TS;
    match s {
        TS::Draw { hand, rank, percentile } => TrainingScenarioIpc::Draw {
            hand: cards_to_ipc(hand.iter().copied()),
            rank,
            percentile,
        },
        TS::Board {
            hole_cards,
            board,
            best_hand,
            rank,
            percentile,
        } => TrainingScenarioIpc::Board {
            hole_cards: cards_to_ipc(hole_cards.iter().copied()),
            board: cards_to_ipc(board.iter().copied()),
            best_hand: cards_to_ipc(best_hand.iter().copied()),
            rank,
            percentile,
        },
        TS::SplitPot {
            hole_cards,
            board,
            best_high_hand,
            high_rank,
            high_percentile,
            low_qualifies,
            best_low_hand,
            low_rank,
            low_percentile,
        } => TrainingScenarioIpc::SplitPot {
            hole_cards: cards_to_ipc(hole_cards.iter().copied()),
            board: cards_to_ipc(board.iter().copied()),
            best_high_hand: cards_to_ipc(best_high_hand.iter().copied()),
            high_rank,
            high_percentile,
            low_qualifies,
            best_low_hand: best_low_hand.map(|h| cards_to_ipc(h.iter().copied())),
            low_rank,
            low_percentile,
        },
    }
}

#[tauri::command]
fn get_game_training_info(game_id: String) -> Result<GameTrainingInfo, String> {
    let variant = GameVariant::from_game_id(&game_id);
    let strategy = training::training_strategy(variant);
    Ok(strategy.training_info())
}

#[tauri::command]
fn generate_training_scenario(
    input: TrainingScenarioInputIpc,
) -> Result<TrainingScenarioIpc, String> {
    let variant = GameVariant::from_game_id(&input.game_id);
    let strategy = training::training_strategy(variant);

    let held_cards: Vec<Card> = input
        .held_cards
        .iter()
        .map(card_input_to_card)
        .collect::<Result<Vec<_>, _>>()?;
    let dead_cards: Vec<Card> = input
        .dead_cards
        .iter()
        .map(card_input_to_card)
        .collect::<Result<Vec<_>, _>>()?;

    let seed = input
        .seed
        .unwrap_or_else(|| rand::rngs::OsRng.next_u64());

    let scenario = strategy
        .generate_scenario(&held_cards, &dead_cards, input.sim_count, seed)
        .map_err(|e| e.to_string())?;

    Ok(training_scenario_to_ipc(scenario))
}

const STORE_PATH: &str = "store.json";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let storage = StorageState(Arc::new(StoreBackend::new(
                app.handle().clone(),
                STORE_PATH,
            )));
            app.store(STORE_PATH).map_err(|e| e.to_string())?;
            app.manage(storage);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_simulation,
            get_games,
            get_training_config,
            get_ui_config,
            get_preferences,
            set_preferences,
            get_game_training_info,
            generate_training_scenario,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
