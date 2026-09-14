mod abilities;
mod actions;
mod agent;
mod agents;
mod cards;
mod faction;
mod game;
mod player;
mod util;

use agents::{PassBot10000, UserCLI};
use game::Game;

fn main() {
    let mut game = Game::new();
    let agent1 = PassBot10000 {};

    let agent1_cli = UserCLI {
        recent_messages: Vec::new(),
    };

    let agent2 = UserCLI {
        recent_messages: Vec::new(),
    };

    // game.run_game(&mut [Box::new(agent1), Box::new(agent2)]);
    game.run_game(&mut [Box::new(agent1_cli), Box::new(agent2)]);
    // println!("{}", &CARDS[&CardNamed::Cutter]);
}
