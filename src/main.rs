mod abilities;
mod actions;
mod agent;
mod agents;
mod cards;
mod faction;
mod game;
mod player;
mod util;

use std::io::stdin;

use agents::{BasicBot, PassBot10000, UserCLI};
use game::Game;

fn main() {
    let mut game = Game::new();
    let agent1 = BasicBot {};

    let mut name = String::new();

    // println!("Enter name 1: ");
    // stdin().read_line(&mut name).unwrap();
    // let agent1_cli = UserCLI::new(name.clone());

    println!("Enter name 2: ");
    stdin().read_line(&mut name).unwrap();
    let agent2 = UserCLI::new(name);

    game.run_game(&mut [Box::new(agent1), Box::new(agent2)]);
    // game.run_game(&mut [Box::new(agent1_cli), Box::new(agent2)]);
    // println!("{}", &CARDS[&CardNamed::Cutter]);
}
