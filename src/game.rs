use std::{collections::HashSet, fmt::Display};

use rand::{RngExt, rngs::ThreadRng, seq::SliceRandom};

use crate::actions::CardAction;
use crate::actions::CombatTarget::{Enemy, EnemyBase};
use crate::actions::PlayerAction::{EndTurn, PlayCard};
use crate::actions::{AskContext, PlayerAction};
use crate::agent::Agent;
use crate::cards::{CARDS, CardNamed, STARTER_GAME_DECK, effect};
use crate::effects::Effect;
use crate::effects::Resource::{Authority, Combat, Trade};
use crate::effects::{Condition, PileFlag};
use crate::player::{InPlayCard, Player};

pub struct Game {
    pub players: [Player; 2],
    pub turn_number: u32,
    pub rng: ThreadRng,
    pub deck: Vec<CardNamed>,
    pub shop: [Option<CardNamed>; 5],
}

pub struct IllegalAction;

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "player {}:", self.turn_number % 2 + 1)?;
        writeln!(f, "{}", self.players[(self.turn_number as usize + 1) % 2])?;
        writeln!(f)?;
        writeln!(f, "turn: {}", self.turn_number)?;
        writeln!(f, "trade deck: {} cards", self.deck.len())?;
        writeln!(
            f,
            "shop: {}",
            self.shop
                .iter()
                .map(|c| match c {
                    Some(c) => CARDS[c].name_only(),
                    None => "-".to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        writeln!(f)?;
        writeln!(f, "player {}:", (self.turn_number + 1) % 2 + 1)?;
        writeln!(f, "{}", self.players[(self.turn_number as usize) % 2])
    }
}

impl Game {
    pub fn new() -> Game {
        let mut player1 = Player::default();
        let mut player2 = Player::default();
        let mut rng = rand::rng();

        let turn_number = rng.random_range(0..=1);
        let mut deck = STARTER_GAME_DECK.clone();
        deck.shuffle(&mut rng);
        let mut deck_iter = deck.into_iter();

        let shop = [
            deck_iter.next(),
            deck_iter.next(),
            deck_iter.next(),
            deck_iter.next(),
            deck_iter.next(),
        ];

        if turn_number == 0 {
            player1.draw_cards(3, &mut rng);
            player2.draw_cards(5, &mut rng);
        } else {
            player1.draw_cards(5, &mut rng);
            player2.draw_cards(3, &mut rng);
        }

        Self {
            players: [player1, player2],
            turn_number,
            rng,
            deck: deck_iter.collect(),
            shop,
        }
    }

    pub fn game_ended(&self) -> bool {
        self.players.iter().any(|p| p.authority <= 0)
    }

    pub fn run_game(&mut self, agents: &mut [Box<dyn Agent>; 2]) {
        loop {
            let actor = self.turn_number as usize % 2;

            for base in &self.players[actor].bases_in_play() {
                self.resolve_effect(
                    agents,
                    actor,
                    *base,
                    &Effect::Sequence(effect(&base.name).clone()),
                );
            }

            while !self.game_ended() {
                let action = agents[actor].choose_action(self, actor);

                if action == PlayerAction::EndTurn {
                    break;
                }

                match self.do_action(agents, action) {
                    Ok(_) => {}
                    Err(_) => {}
                };
            }

            if self.game_ended() {
                break;
            }

            let player = &mut self.players[actor];

            // all ships in play get discarded
            for s in player.ships_in_play() {
                if let Some(name) = player.remove_from_play(s.id) {
                    player.discard_pile.push(name);
                }
            }

            // all in hand get discarded
            player.discard_pile.extend(player.hand.drain(..));

            player.draw_cards(5, &mut self.rng);

            #[cfg(feature = "reset_resources")]
            {
                player.combat = 0;
                player.trade = 0;
            }

            self.turn_number += 1;
        }
    }
    fn do_action(
        &mut self,
        agents: &mut [Box<dyn Agent>; 2],

        action: PlayerAction,
    ) -> Result<(), IllegalAction> {
        let actor = self.turn_number as usize % 2;

        let [p1, p2] = &mut self.players;
        let (player, enemy) = if actor == 0 { (p1, p2) } else { (p2, p1) };

        match action {
            PlayCard(hand_index) => {
                if hand_index >= player.hand.len() {
                    return Err(IllegalAction);
                }

                self.play_from_hand(agents, actor, hand_index);
            }
            PlayerAction::BuyCard(card_index) => {
                let Some(target_card) = self.shop[card_index] else {
                    return Err(IllegalAction);
                };
                let target_cost = CARDS[&target_card].cost;

                if target_cost > player.trade {
                    return Err(IllegalAction);
                }

                player.trade -= target_cost;
                self.shop[card_index] = self.deck.pop(); // Draw new card, if no card its None anyways

                player.discard_pile.push(target_card); // TODO: next card from shop to top of deck
            }
            PlayerAction::Card(card_index, card_action) => {
                let Some(&target_card) = player.in_play.get(card_index) else {
                    return Err(IllegalAction);
                };

                match card_action {
                    CardAction::Scrap => {
                        let Some(Effect::ScrapAbility(inner)) = CARDS[&target_card.name]
                            .effects
                            .iter()
                            .find(|e| matches!(e, Effect::ScrapAbility(_)))
                        else {
                            return Err(IllegalAction);
                        };

                        player.remove_from_play(target_card.id);

                        self.resolve_effect(agents, actor, target_card, inner);
                    }
                    CardAction::EngageEffect => todo!(),
                }
            }
            PlayerAction::SpendCombat(combat_target) => {
                let enemy_has_outposts = !enemy.outposts_in_play().is_empty();
                match combat_target {
                    Enemy => {
                        if enemy_has_outposts {
                            // Cannot target enemy if has outpost
                            return Err(IllegalAction);
                        }
                        enemy.authority = enemy.authority.saturating_sub(player.combat);
                        player.combat = 0;
                    }
                    EnemyBase(base_index) => {
                        let Some(&target_base) = enemy.bases_in_play().get(base_index) else {
                            return Err(IllegalAction);
                        };

                        let target_base_card = &CARDS[&target_base.name];

                        if enemy_has_outposts && target_base_card.is_outpost() {
                            // Cannot attack non-outpost bases if has outpost
                            return Err(IllegalAction);
                        }

                        if player.combat < target_base_card.get_base_defense().unwrap() {
                            // Not enough combat to attack base. Just waists combat.
                            return Err(IllegalAction);
                        }

                        if let Some(name) = enemy.remove_from_play(target_base.id) {
                            enemy.discard_pile.push(name);
                        }
                    }
                }
            }

            PlayerAction::PlayAll => {
                let mut n = self.players[actor].hand.len();

                while n > 0 && !self.players[actor].hand.is_empty() {
                    self.play_from_hand(agents, actor, 0);
                    n -= 1;
                }
            }
            EndTurn => todo!(),
        }

        Ok(())
    }

    fn play_from_hand(
        &mut self,
        agents: &mut [Box<dyn Agent>; 2],
        actor: usize,
        hand_index: usize,
    ) {
        let player = &mut self.players[actor];
        let played_card = player.hand.remove(hand_index);
        let instance = player.play_card(played_card);

        self.resolve_effect(
            agents,
            actor,
            InPlayCard {
                id: instance,
                name: played_card,
            },
            &Effect::Sequence(CARDS[&played_card].effects.clone()),
        );

        let player = &mut self.players[actor];
        let mut effect_queue = Vec::new();
        let faction_count = player.faction_count();

        player
            .pending_ally_effects
            .retain(|(in_play_card, faction, effect)| {
                if faction_count[faction] >= 2 {
                    effect_queue.push((*in_play_card, effect.clone()));
                    return false; // dont keep
                }
                // keep
                true
            });

        for (in_play_card, effect) in effect_queue {
            self.resolve_effect(agents, actor, in_play_card, &effect);
        }
    }

    fn resolve_effect(
        &mut self,
        agents: &mut [Box<dyn Agent>; 2],
        actor: usize,
        source: InPlayCard,
        effect: &Effect,
    ) {
        let ctx = AskContext {
            actor,
            source,
            source_effect: effect,
        };

        match effect {
            Effect::Resource(resource, count) => {
                let player = &mut self.players[actor];
                match resource {
                    Authority => player.authority += count,
                    Combat => player.combat += count,
                    Trade => player.trade += count,
                }
            }
            Effect::Sequence(effects) => {
                for e in effects {
                    self.resolve_effect(agents, actor, source, e);
                }
            }
            Effect::If(condition, inner) => {
                if self.evaluate_condition(actor, condition) {
                    self.resolve_effect(agents, actor, source, inner);
                }
            }
            // Ally abilities become available for at-will use once queued here;
            // they aren't resolved immediately like a normal effect.
            Effect::Ally(faction, inner) => {
                self.players[actor].pending_ally_effects.push((
                    source,
                    *faction,
                    (**inner).clone(),
                ));
            }
            // Held abilities: never walked during normal play, only looked up
            // on demand (ScrapInPlay action, or the PlayShip trigger scan) —
            // so they're intentional no-ops here.
            Effect::ScrapAbility(_) | Effect::Trigger(_, _) => {}
            Effect::May(inner) => {
                if agents[actor].ask_yes_no(self, &ctx) {
                    self.resolve_effect(agents, actor, source, inner);
                }
            }
            Effect::Scrap(pile, count) => {
                let player = &mut self.players[actor];
                let mut max_count = 0;

                if pile.contains(PileFlag::HAND) {
                    max_count += player.hand.len();
                }

                if pile.contains(PileFlag::DISCARD_PILE) {
                    max_count += player.discard_pile.len();
                }
                let new_count = (*count).min(max_count);

                if new_count == 0 {
                    return; // nothing to do
                }

                loop {
                    let choices = agents[actor].ask_cards_from_pile(self, &ctx, *pile, new_count);
                    let player = &mut self.players[actor];

                    if choices.iter().any(|&(p, index)| {
                        p == PileFlag::all()
                            || p == PileFlag::empty()
                            || p == PileFlag::HAND && index >= player.hand.len()
                            || p == PileFlag::DISCARD_PILE && index >= player.discard_pile.len()
                    }) {
                        continue;
                    }

                    let hand_choices = choices
                        .iter()
                        .filter_map(|(p, index)| (*p == PileFlag::HAND).then_some(*index))
                        .collect::<Vec<_>>();
                    let discard_pile_choices = choices
                        .iter()
                        .filter_map(|(p, index)| (*p == PileFlag::DISCARD_PILE).then_some(*index))
                        .collect::<Vec<_>>();

                    if hand_choices.len()
                        != hand_choices
                            .iter()
                            .copied()
                            .collect::<HashSet<usize>>()
                            .len()
                    {
                        continue;
                    }

                    if discard_pile_choices.len()
                        != discard_pile_choices
                            .iter()
                            .copied()
                            .collect::<HashSet<usize>>()
                            .len()
                    {
                        continue;
                    }

                    player.hand = player
                        .hand
                        .iter()
                        .copied()
                        .enumerate()
                        .filter_map(|(i, c)| (!hand_choices.contains(&i)).then_some(c))
                        .collect();

                    player.discard_pile = player
                        .discard_pile
                        .iter()
                        .copied()
                        .enumerate()
                        .filter_map(|(i, c)| (!discard_pile_choices.contains(&i)).then_some(c))
                        .collect();

                    break;
                }
            }
            Effect::Or(effect, effect1) => {
                if agents[actor].ask_yes_no(self, &ctx) {
                    self.resolve_effect(agents, actor, source, effect);
                } else {
                    self.resolve_effect(agents, actor, source, effect1);
                }
            }
            Effect::Draw(amount) => {
                let count = amount.compute(self);
                self.players[actor].draw_cards(count, &mut self.rng);
            }
            Effect::AquireShipForFree {
                to_top_of_deck,
                max_cost,
            } => todo!("select card from shop"),
            Effect::Discard(count) => todo!("select card from hand"),
            Effect::DestroyTargetBase => todo!("select enemy base in play"),
            Effect::ScrapCardInRow => todo!("select card from shop"),
            Effect::OpponentDiscards => todo!("no actions"),
            Effect::CopyPlayedShip => todo!("chose played shi"),
            Effect::NextAcquiredShipToTopOfDeck => todo!("no action"),
        }
    }

    fn evaluate_condition(&self, actor: usize, condition: &Condition) -> bool {
        match condition {
            Condition::And(condition, condition1) => {
                self.evaluate_condition(actor, condition)
                    && self.evaluate_condition(actor, condition1)
            }
            Condition::BaseCountAtLeast(count) => {
                self.players[actor]
                    .in_play
                    .iter()
                    .filter(|c| CARDS[&c.name].is_base())
                    .count()
                    >= *count
            }
        }
    }
}
