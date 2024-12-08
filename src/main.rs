use postflop_solver::*;
use serde_json::Value;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Activate debug mode
    // // short and long flags (-d, --debug) will be deduced from the field's name
    // #[structopt(short, long)]
    // debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

// TODO: refactor using trait
fn extract_str(data: &Value, keys: &Vec<&str>) -> String {
    let mut tmp = data;
    for key in keys {
        tmp = tmp.get(key).unwrap();
        println!("Look data: {:?}", key);
    }

    return String::from(tmp.as_str().unwrap());
}

fn extract_float(data: &Value, keys: &Vec<&str>) -> f64 {
    let mut tmp = data;
    for key in keys {
        tmp = tmp.get(key).unwrap();
    }

    return tmp.as_f64().unwrap();
}

fn extract_i32(data: &Value, keys: &Vec<&str>) -> i32 {
    let mut tmp = data;
    for key in keys {
        tmp = tmp.get(key).unwrap();
    }

    return tmp.as_i64().unwrap() as i32;
}

fn main() {
    let opt = Opt::from_args();

    let contents = std::fs::read_to_string(opt.input).expect("Should read file");
    let data: Value = serde_json::from_str(&contents).expect("Error while parsing json");
    let oop_range = extract_str(&data, &vec!["oop_range"]);
    let ip_range = extract_str(&data, &vec!["ip_range"]);
    let mut initial_state = BoardState::Flop;
    let flop = extract_str(&data, &vec!["public_card", "flop"]);
    let turn = extract_str(&data, &vec!["public_card", "turn"]);
    let river = extract_str(&data, &vec!["public_card", "river"]);
    let mut card_config = CardConfig {
        range: [oop_range.parse().unwrap(), ip_range.parse().unwrap()],
        flop: flop_from_str(&flop).unwrap(),
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };
    if turn != "" {
        card_config.turn = card_from_str(&turn).unwrap();
        initial_state = BoardState::Turn;
    }
    if river != "" {
        card_config.river = card_from_str(&river).unwrap();
        initial_state = BoardState::River;
    }
    let bet_sizes = BetSizeOptions::try_from(("60%, e, a", "2.5x")).unwrap();
    let river_donk_sizes_str = extract_str(&data, &vec!["tree_config", "river_donk_sizes"]);

    let tree_config = TreeConfig {
        initial_state: initial_state,
        starting_pot: extract_i32(&data, &vec!["tree_config", "starting_pot"]),
        effective_stack: extract_i32(&data, &vec!["tree_config", "effective_stack"]),
        rake_rate: extract_float(&data, &vec!["tree_config", "rake_rate"]),
        rake_cap: extract_float(&data, &vec!["tree_config", "rake_cap"]),
        flop_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()], // [OOP, IP]
        turn_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        river_bet_sizes: [bet_sizes.clone(), bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: Some(DonkSizeOptions::try_from(river_donk_sizes_str.as_str()).unwrap()),
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    let action_tree = ActionTree::new(tree_config).unwrap();
    let mut game = PostFlopGame::with_config(card_config, action_tree).unwrap();
    game.allocate_memory(false);
    let max_num_iterations = 1;
    let target_exploitability = game.tree_config().starting_pot as f32 * 0.005; // 0.5% of the pot
    let exploitability = solve(&mut game, max_num_iterations, target_exploitability, true);
    println!("Exploitability: {:.2}", exploitability);
}
