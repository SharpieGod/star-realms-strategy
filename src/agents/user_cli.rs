use std::io::stdin;

use crate::actions::AskContext;

use crate::abilities::{Ability, PileFlag};
use crate::actions::CombatTarget::{Enemy, EnemyBase};
use crate::actions::PlayerAction::{self, BuyCard, EndTurn, PlayCard, SpendCombat};
use crate::agent::Agent;
use crate::cards::CARDS;
use crate::cards::CardNamed::{self, Scout};
use crate::game::Game;
use crate::util::clear_console;

pub struct UserCLI {
    pub recent_messages: Vec<String>,
}

impl UserCLI {
    fn print_recent_messages(&mut self) {
        if !self.recent_messages.is_empty() {
            println!(
                "\n------------\n\n{}\n\n------------",
                self.recent_messages.join("\n\n")
            );
        }
        self.recent_messages.clear();
    }

    fn print_game_state(&self, game: &Game, actor: usize) {
        let player = &game.players[actor];
        let enemy = &game.players[(actor + 1) % 2];
        println!("enemy authority: {}", enemy.authority);
        println!(
            "enemy bases: {}",
            enemy
                .in_play
                .iter()
                .filter(|c| CARDS[&c.name].is_base())
                .enumerate()
                .map(|(i, c)| format!("{i}:{}", c.name))
                .collect::<Vec<String>>()
                .join(", ")
        );

        println!("----------------------------------\n");

        println!(
            "shop:\n{}",
            game.shop
                .iter()
                .enumerate()
                .map(|(i, c)| c
                    .map(|c| format!("{i}:{}", c.with_cost()))
                    .unwrap_or_else(|| " - ".to_string()))
                .collect::<Vec<String>>()
                .join(", ")
        );

        println!();

        println!(
            "authority: {}, combat: {}, trade: {}",
            player.authority, player.combat, player.trade
        );

        println!("deck: {} cards", player.personal_deck.len());

        println!(
            "bases in play: {}",
            player
                .in_play
                .iter()
                .filter(|c| CARDS[&c.name].is_base())
                .enumerate()
                .map(|(i, c)| format!("{i}:{}", c.name))
                .collect::<Vec<String>>()
                .join(", ")
        );
        println!(
            "ships in play: {}",
            player
                .in_play
                .iter()
                .filter(|c| !CARDS[&c.name].is_base())
                .enumerate()
                .map(|(i, c)| format!("{i}:{}", c.name))
                .collect::<Vec<String>>()
                .join(", ")
        );

        println!(
            "discard pile: {}",
            player
                .discard_pile
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{i}:{}", c))
                .collect::<Vec<String>>()
                .join(", ")
        );
        println!();

        println!(
            "hand: {}",
            player
                .hand
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{i}:{}", c))
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
}

impl Agent for UserCLI {
    fn choose_action(&mut self, game: &Game, actor: usize) -> PlayerAction {
        let player = &game.players[actor];
        let enemy = &game.players[(actor + 1) % 2];

        loop {
            clear_console();
            // println!("{game}");
            self.print_game_state(game, actor);
            self.print_recent_messages();

            let mut s = String::new();
            stdin().read_line(&mut s).unwrap();

            s = s.trim().to_string();

            // PlayCard(_) play index
            // PlayerAction::BuyCard(_) buy index
            // PlayerAction::Card(_, card_action) index {card action}
            // PlayerAction::SpendCombat(combat_target) damage {target}
            // PlayerAction::EndTurn turn/end turn/any invalid input

            let tokens = s.split_whitespace().collect::<Vec<&str>>();

            match tokens.get(0).copied().unwrap_or_default() {
                "play" if tokens.get(1).copied().unwrap_or_default() == "all" => {
                    return PlayerAction::PlayAll;
                }
                c if (c == "play" || c == "buy") => {
                    let Ok(index) = tokens.get(1).copied().unwrap_or_default().parse::<usize>()
                    else {
                        continue;
                    };

                    return match c {
                        "play" => PlayCard(index),
                        "buy" => BuyCard(index),
                        _ => unreachable!(),
                    };
                }

                "scrap"
                    if tokens
                        .get(2)
                        .copied()
                        .unwrap_or_default()
                        .parse::<usize>()
                        .is_ok() =>
                {
                    let index = tokens
                        .get(2)
                        .copied()
                        .unwrap_or_default()
                        .parse::<usize>()
                        .unwrap();

                    match tokens.get(1).copied().unwrap_or_default() {
                        "base" => {
                            return PlayerAction::Scrap {
                                is_base: true,
                                index,
                            };
                        }
                        "ship" => {
                            return PlayerAction::Scrap {
                                is_base: false,
                                index,
                            };
                        }
                        _ => {
                            continue;
                        }
                    }
                }

                "damage" | "d" => match tokens.get(1).copied().unwrap_or_default() {
                    "base" => {
                        let Ok(index) = tokens.get(2).copied().unwrap_or_default().parse::<usize>()
                        else {
                            continue;
                        };

                        return SpendCombat(EnemyBase(index));
                    }
                    "enemy" => return SpendCombat(Enemy),
                    _ => {}
                },

                "info" | "i" => {
                    let Ok(index) = tokens.get(2).unwrap_or(&"").parse::<usize>() else {
                        continue;
                    };

                    let player_ships = player
                        .ships_in_play()
                        .iter()
                        .map(|b| b.name)
                        .collect::<Vec<CardNamed>>();

                    let player_bases = player
                        .bases_in_play()
                        .iter()
                        .map(|b| b.name)
                        .collect::<Vec<CardNamed>>();

                    let enemy_bases = enemy
                        .bases_in_play()
                        .iter()
                        .map(|b| b.name)
                        .collect::<Vec<CardNamed>>();

                    clear_console();
                    println!(
                        "{}",
                        CARDS[match *tokens.get(1).unwrap_or(&"") {
                            "hand" | "h" => {
                                player.hand.get(index)
                            }
                            "play" | "p" | "ip" | "ships" => {
                                player_ships.get(index)
                            }
                            "bases" | "b" => {
                                player_bases.get(index)
                            }
                            "shop" | "s" => {
                                game.shop.get(index).unwrap_or(&None).as_ref()
                            }
                            "enemybases" | "eb" => {
                                enemy_bases.get(index)
                            }
                            "enemyhand" | "eh" => {
                                enemy.hand.get(index)
                            }

                            _ => {
                                None
                            }
                        }
                        .unwrap_or(&Scout)]
                    );

                    stdin().read_line(&mut String::new()).unwrap();
                }
                "pass" | "turn" | "end turn" => return EndTurn,
                _ => {}
            }
        }
    }

    fn ask_yes_no(&mut self, game: &Game, ctx: &AskContext) -> bool {
        let out;
        let ctx_message = format!("{}: {}", ctx.source.name, ctx.source_ability);

        let (prompt, is_true, is_false): (String, fn(&str) -> bool, fn(&str) -> bool) =
            match ctx.source_ability {
                Ability::Or(_, _) => (
                    format!("{ctx_message}\nfirst or second?",),
                    |x| matches!(x, "f" | "first" | "l" | "left" | "0"),
                    |x| matches!(x, "s" | "second" | "1" | "r" | "right"),
                ),
                _ => (
                    format!("{ctx_message}\nyes or no?",),
                    |x| matches!(x, "y" | "yes"),
                    |x| matches!(x, "n" | "no"),
                ),
            };

        loop {
            clear_console();
            self.print_game_state(game, game.turn_number as usize % 2);

            self.print_recent_messages();
            println!("\n");

            println!("{prompt}");

            let mut s = String::new();
            stdin().read_line(&mut s).unwrap();

            let x = &s.trim().to_lowercase();

            if is_true(x) {
                out = true;
                break;
            } else if is_false(x) {
                out = false;
                break;
            }
        }

        match ctx.source_ability {
            Ability::Or(ability, ability1) => {
                self.recent_messages.push(format!(
                    "{ctx_message}\nselected {{{}}}",
                    if out { ability } else { ability1 }
                ));
            }
            _ => {
                self.recent_messages.push(format!(
                    "{ctx_message}\nselected {}",
                    if out { "yes" } else { "no" }
                ));
            }
        }

        out
    }

    fn ask_shop_card(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize {
        clear_console();
        loop {
            self.print_game_state(game, game.players.len() % 2);
            self.print_recent_messages();

            println!(
                "{}: {}\nwhich card from shop\neligible: {}",
                ctx.source.name,
                ctx.source_ability,
                eligible
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            let mut s = String::new();
            stdin().read_line(&mut s).unwrap();

            clear_console();

            let Ok(index) = s.trim().parse::<usize>() else {
                println!("not valid index");
                continue;
            };

            if !eligible.contains(&index) {
                println!("not valid index");
                continue;
            }

            return index;
        }
    }

    fn ask_enemy_base(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize {
        todo!()
    }

    fn ask_played_ship(&mut self, game: &Game, ctx: &AskContext) -> usize {
        todo!()
    }

    fn ask_cards_from_pile(
        &mut self,
        game: &Game,
        ctx: &AskContext,
        pile: PileFlag,
        count: usize,
    ) -> Vec<(PileFlag, usize)> {
        clear_console();
        let mut out: Vec<(PileFlag, usize)> = Vec::new();
        let player = &game.players[ctx.actor];

        let mut selected_cards = Vec::new();

        while out.len() < count as usize {
            self.print_recent_messages();
            println!("\n");

            if pile.contains(PileFlag::HAND) {
                println!(
                    "hand: {}",
                    player
                        .hand
                        .iter()
                        .enumerate()
                        .map(|(i, c)| if !out.contains(&(PileFlag::HAND, i)) {
                            format!("{i}:{c}")
                        } else {
                            format!("({c})")
                        })
                        .collect::<Vec<String>>()
                        .join(" ")
                );
            }

            if pile.contains(PileFlag::DISCARD_PILE) {
                println!(
                    "discard pile: {}",
                    player
                        .discard_pile
                        .iter()
                        .enumerate()
                        .map(|(i, c)| if !out.contains(&(PileFlag::DISCARD_PILE, i)) {
                            format!("{i}:{c}")
                        } else {
                            format!("({c})")
                        })
                        .collect::<Vec<String>>()
                        .join(" ")
                );
            }

            println!("{}: {}", ctx.source.name, ctx.source_ability);
            println!("choose card {}/{count} from {pile}\n", out.len() + 1);

            let mut s = String::new();

            stdin().read_line(&mut s).unwrap();
            s = s.trim().to_string();
            let tokens = s.split_whitespace().collect::<Vec<&str>>();

            clear_console();
            if pile == PileFlag::HAND.union(PileFlag::DISCARD_PILE) {
                let Ok(index) = tokens.get(1).copied().unwrap_or_default().parse::<usize>() else {
                    println!(
                        "invalid usize {:?}",
                        tokens.get(1).copied().unwrap_or_default()
                    );

                    continue;
                };

                match tokens.get(0).copied().unwrap_or_default() {
                    "hand" | "h" => {
                        if !pile.contains(PileFlag::HAND) {
                            println!("invalid pile. only allowed {pile}");
                            continue;
                        }

                        if index >= player.hand.len() {
                            println!(
                                "max index in hand is {}. invalid index",
                                player.hand.len() as i32 - 1
                            );

                            continue;
                        }

                        if out.iter().any(|(f, i)| *f == PileFlag::HAND && *i == index) {
                            println!("already chose this index. invalid index",);
                            continue;
                        }

                        out.push((PileFlag::HAND, index));
                        selected_cards.push(player.hand.get(index));
                    }
                    "discardpile" | "pile" | "dp" | "d" => {
                        if !pile.contains(PileFlag::DISCARD_PILE) {
                            println!("invalid pile. only allowed {pile}");
                            continue;
                        }
                        if index >= player.discard_pile.len() {
                            println!(
                                "max index in hand is {}. invalid index",
                                player.discard_pile.len() as i32 - 1
                            );
                            continue;
                        }

                        if out
                            .iter()
                            .any(|(f, i)| *f == PileFlag::DISCARD_PILE && *i == index)
                        {
                            println!("already chose this index. invalid index",);
                            continue;
                        }

                        selected_cards.push(player.discard_pile.get(index));
                        out.push((PileFlag::DISCARD_PILE, index))
                    }
                    _ => {
                        println!("invalid pile selector. (hand|discardpile)")
                    }
                }
            } else {
                let Ok(index) = tokens.get(0).copied().unwrap_or_default().parse::<usize>() else {
                    println!(
                        "invalid usize {:?}",
                        tokens.get(0).copied().unwrap_or_default()
                    );

                    continue;
                };

                let player_pile = if pile == PileFlag::HAND {
                    &player.hand
                } else {
                    &player.discard_pile
                };

                let max_len = if pile == PileFlag::HAND {
                    player.hand.len()
                } else {
                    player.discard_pile.len()
                };

                if index >= max_len {
                    println!("max index is {}. invalid index", max_len - 1);
                    continue;
                }

                if out.iter().any(|(f, i)| *f == pile && *i == index) {
                    println!("already chose this index. invalid index",);
                    continue;
                }

                selected_cards.push(player_pile.get(index));

                out.push((pile, index));
            }
        }

        self.recent_messages.push(format!(
            "{}: {}\nselected {}",
            ctx.source.name,
            ctx.source_ability,
            selected_cards
                .iter()
                .map(|c| c.map(|c| c.to_string()).unwrap_or_default())
                .collect::<Vec<String>>()
                .join(", ")
        ));

        out
    }
}
