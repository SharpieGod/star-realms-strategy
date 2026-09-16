mod abilities;
mod actions;
mod agent;
mod agents;
mod cards;
mod faction;
mod game;
mod player;
mod util;

use std::{
    io::stdin,
    time::{Duration, Instant},
};

use agents::{BasicBot, PassBot10000, UserCLI};
use game::Game;

use crate::agent::Agent;

fn main() {
    // let mut cli = UserCLI::new("Ethan".to_string());
    // let mut basic = BasicBot {};

    // let mut game = Game::new();

    // game.run_game(&mut [Box::new(cli), Box::new(basic)]);

    let mut last_check = Instant::now();

    let mut player_wins = [0, 0];
    const CHECK_INTERVAL: usize = 100;
    let mut count = 0;

    for i in 0..200000 {
        if i % CHECK_INTERVAL == 0 {
            let elapsed = last_check.elapsed();
            if elapsed >= Duration::from_millis(500) {
                let rate = count as f64 / elapsed.as_secs_f64();
                println!("Estimated Rate: {:.2} iter/sec", rate);

                // Reset for the next window
                last_check = Instant::now();
                count = 0;
            }
        }

        count += 1;

        let mut game = Game::new();
        let agent1 = BasicBot {};
        let agent2 = BasicBot {};
        let mut agents: [Box<dyn Agent>; 2] = [Box::new(agent1), Box::new(agent2)];
        let player_who_went_first = game.turn_number as usize % 2;

        game.run_game(&mut agents);

        if game.players[player_who_went_first].authority != 0 {
            player_wins[0] += 1;
        } else {
            player_wins[1] += 1;
        }

        drop(agents)
    }

    let sum = player_wins[0] + player_wins[1];

    println!(
        "going first {:.3}%\ngoing second {:.3}%",
        player_wins[0] as f32 / sum as f32 * 100.0,
        player_wins[1] as f32 / sum as f32 * 100.0
    );
}
