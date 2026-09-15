use std::{collections::HashSet, fmt::Display};

use rand::{RngExt, rngs::ThreadRng, seq::SliceRandom};

use crate::abilities::Ability::{self, Scrap};
use crate::abilities::Resource::{Authority, Combat, Trade};
use crate::abilities::{Condition, PileFlag};
use crate::actions::CombatTarget::{Enemy, EnemyBase};
use crate::actions::PlayerAction::{BuyCard, EndTurn, PlayCard, SpendCombat};
use crate::actions::{AskContext, PlayerAction};
use crate::agent::Agent;
use crate::cards::{CARDS, CardNamed, CardType, STARTER_GAME_DECK, ability};
use crate::player::{CardInstanceId, InPlayCard, Player};

pub struct Game {
    pub players: [Player; 2],
    pub turn_number: u32,
    pub rng: ThreadRng,
    pub deck: Vec<CardNamed>,
    pub shop: [Option<CardNamed>; 6],
    pub next_aquired_ship_to_top_of_deck: bool,
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
            Some(CardNamed::Explorer),
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
            next_aquired_ship_to_top_of_deck: false,
        }
    }

    pub fn legal_moves(&self, actor: usize) -> Vec<PlayerAction> {
        let mut out = Vec::new();
        let player = &self.players[actor];
        let enemy = &self.players[(actor + 1) % 2];

        if !player.hand.is_empty() {
            out.push(PlayerAction::PlayAll);

            out.extend(
                player
                    .hand
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(i, _)| PlayCard(i))
                    .collect::<Vec<PlayerAction>>(),
            );
        }

        out.extend(self.shop.iter().enumerate().filter_map(|(i, c)| {
            c.is_some_and(|c| CARDS[&c].cost <= player.trade)
                .then_some(BuyCard(i))
        }));

        let enemy_has_outposts = !enemy.outposts_in_play().is_empty();

        if !enemy_has_outposts && player.combat > 0 {
            out.push(SpendCombat(Enemy));
        }

        out.extend(
            enemy
                .bases_in_play()
                .iter()
                .enumerate()
                .filter_map(|(i, b)| {
                    (CARDS[&b.name].get_base_defense().unwrap() <= player.combat
                        && (enemy_has_outposts && CARDS[&b.name].is_outpost()
                            || !enemy_has_outposts))
                        .then_some(SpendCombat(EnemyBase(i)))
                }),
        );

        out.push(EndTurn);

        out.extend(
            enemy
                .ships_in_play()
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    s.name.has_scrap_ability().then_some(PlayerAction::Scrap {
                        is_base: false,
                        index: i,
                    })
                }),
        );

        out.extend(
            enemy
                .bases_in_play()
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    s.name.has_scrap_ability().then_some(PlayerAction::Scrap {
                        is_base: true,
                        index: i,
                    })
                }),
        );

        out
    }
    pub fn game_ended(&self) -> bool {
        self.players.iter().any(|p| p.authority <= 0)
    }

    pub fn run_game(&mut self, agents: &mut [Box<dyn Agent>; 2]) {
        loop {
            let actor = self.turn_number as usize % 2;

            let base_ids: Vec<(CardInstanceId, CardNamed)> = self.players[actor]
                .bases_in_play()
                .into_iter()
                .map(|base| (base.id, base.name))
                .collect();

            for (id, name) in base_ids {
                self.resolve_ability(
                    agents,
                    actor,
                    id,
                    &Ability::Sequence(ability(&name).clone()),
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
            let ship_ids: Vec<CardInstanceId> =
                player.ships_in_play().into_iter().map(|s| s.id).collect();
            for id in ship_ids {
                if let Some(name) = player.remove_from_play(id) {
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

            self.next_aquired_ship_to_top_of_deck = false;
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
                if target_card != CardNamed::Explorer {
                    // Don't replace explorer when buying it
                    self.shop[card_index] = self.deck.pop(); // Draw new card, if no card its None anyways
                }

                if !self.next_aquired_ship_to_top_of_deck {
                    player.discard_pile.push(target_card);
                } else {
                    player.personal_deck.push(target_card);
                }
            }
            PlayerAction::Scrap { is_base, index } => {
                let player = &self.players[actor];

                let Some(&target_card) = if is_base {
                    player.bases_in_play()
                } else {
                    player.ships_in_play()
                }
                .get(index) else {
                    return Err(IllegalAction);
                };

                let target_id = target_card.id;
                let name = target_card.copied_card.unwrap_or(target_card.name);

                let Some(Ability::ScrapAbility(inner)) = ability(&name)
                    .iter()
                    .find(|e| matches!(e, Ability::ScrapAbility(_)))
                else {
                    return Err(IllegalAction);
                };

                self.resolve_ability(agents, actor, target_id, inner);

                let player = &mut self.players[actor];
                player.remove_from_play(target_id);
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

                        if enemy_has_outposts && !target_base_card.is_outpost() {
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

                        player.combat = player
                            .combat
                            .saturating_sub(target_base_card.get_base_defense().unwrap())
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
            EndTurn => unreachable!(),
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

        self.resolve_card_abilities(agents, actor, instance, played_card);

        let player = &mut self.players[actor];
        let mut ability_queue = Vec::new();
        let faction_count = player.faction_count;

        player
            .pending_ally_abilities
            .retain(|(in_play_card, faction, ability)| {
                if faction_count[*faction as usize] >= 2 {
                    ability_queue.push((in_play_card.clone(), ability.clone()));
                    return false; // dont keep
                }
                // keep
                true
            });

        for (id, ability) in ability_queue {
            let player = &self.players[actor];
            if player.get_card_instance(id).is_none() {
                continue;
            }

            self.resolve_ability(agents, actor, id, &ability);
        }
    }

    fn resolve_card_abilities(
        &mut self,
        agents: &mut [Box<dyn Agent>; 2],
        actor: usize,
        source: CardInstanceId,
        card: CardNamed,
    ) {
        for a in ability(&card) {
            self.resolve_ability(agents, actor, source, a);
        }
    }

    fn resolve_ability(
        &mut self,
        agents: &mut [Box<dyn Agent>; 2],
        actor: usize,
        source: CardInstanceId,
        ability: &Ability,
    ) {
        let ctx = AskContext {
            actor,
            source,
            source_ability: ability,
        };

        match ability {
            Ability::Resource(resource, count) => {
                let player = &mut self.players[actor];
                match resource {
                    Authority => player.authority += count,
                    Combat => player.combat += count,
                    Trade => player.trade += count,
                }
            }
            Ability::Sequence(abilities) => {
                for e in abilities {
                    self.resolve_ability(agents, actor, source, e);
                }
            }
            Ability::If(condition, inner) => {
                if self.evaluate_condition(actor, condition) {
                    self.resolve_ability(agents, actor, source, inner);
                }
            }
            // Ally abilities become available for at-will use once queued here;
            // they aren't resolved immediately like a normal ability.
            Ability::Ally(faction, inner) => {
                self.players[actor].pending_ally_abilities.push((
                    source,
                    *faction,
                    (**inner).clone(),
                ));
            }
            // Held abilities: never walked during normal play, only looked up
            // on demand (ScrapInPlay action, or the PlayShip trigger scan) —
            // so they're intentional no-ops here.
            Ability::ScrapAbility(_) | Ability::Trigger(_, _) => {}
            Ability::May(inner) => {
                if agents[actor].ask_yes_no(self, &ctx) {
                    self.resolve_ability(agents, actor, source, inner);
                }
            }
            Ability::Scrap(pile, count) => {
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
            Ability::Or(ability, ability1) => {
                if agents[actor].ask_yes_no(self, &ctx) {
                    self.resolve_ability(agents, actor, source, ability);
                } else {
                    self.resolve_ability(agents, actor, source, ability1);
                }
            }
            Ability::Draw(amount) => {
                let count = amount.compute(self);
                self.players[actor].draw_cards(count, &mut self.rng);
            }
            Ability::AquireShipForFree {
                to_top_of_deck,
                max_cost,
            } => {
                let eligible = self
                    .shop
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        c.is_some_and(|c| max_cost.is_none_or(|cost| CARDS[&c].cost <= cost))
                            .then_some(i)
                    })
                    .collect::<Vec<usize>>();

                loop {
                    let target_ship = agents[actor].ask_shop_card(self, &ctx, &eligible);
                }
            }
            Ability::Discard(count) => {
                loop {
                    let cards =
                        agents[actor].ask_cards_from_pile(self, &ctx, PileFlag::HAND, *count);

                    let player = &mut self.players[actor];

                    if cards
                        .iter()
                        .any(|&(pile, index)| pile != PileFlag::HAND || index >= player.hand.len())
                    {
                        continue; // not valid pile or invalid index
                    }

                    if cards.len() != cards.iter().copied().collect::<HashSet<_>>().len() {
                        continue; // duplicate items
                    }

                    let mut discarded = Vec::new();

                    player.hand = player
                        .hand
                        .iter()
                        .copied()
                        .enumerate()
                        .filter_map(|(i, c)| {
                            let contains = cards.contains(&(PileFlag::HAND, i));

                            if contains {
                                discarded.push(c);
                                None
                            } else {
                                Some(c)
                            }
                        })
                        .collect();

                    player.discard_pile.append(&mut discarded);

                    break;
                }
            }
            Ability::DestroyTargetBase => {
                let enemy = &self.players[(actor + 1) % 2];

                let eligible = enemy
                    .bases_in_play()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, in_play)| {
                        let has_outposts = !enemy.outposts_in_play().is_empty();

                        let eligible = has_outposts
                            && matches!(
                                CARDS[&in_play.name].card_type,
                                CardType::Base {
                                    defense: _,
                                    is_outpost: true
                                }
                            )
                            || !has_outposts;

                        eligible.then_some(i)
                    })
                    .collect::<Vec<usize>>();

                if eligible.is_empty() {
                    return; // nothing to destroy
                }

                loop {
                    let selected_base_index = agents[actor].ask_enemy_base(self, &ctx, &eligible);

                    if !eligible.contains(&selected_base_index) {
                        continue;
                    }

                    let enemy = &mut self.players[(actor + 1) % 2];

                    let selected_base = enemy.bases_in_play()[selected_base_index].clone();

                    enemy.remove_from_play(selected_base.id);
                    enemy.discard_pile.push(selected_base.name);

                    break;
                }
            }
            Ability::ScrapCardInRow => {
                let eligible = self
                    .shop
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| c.is_some().then_some(i))
                    .collect::<Vec<usize>>();

                loop {
                    let selected_card = agents[actor].ask_shop_card(self, &ctx, &eligible);

                    if !eligible.contains(&selected_card) {
                        continue;
                    }

                    self.shop[selected_card] = self.deck.pop(); // Draw new card, None is valid value.

                    break;
                }
            }
            Ability::OpponentDiscards => {
                let opponent_index = (actor + 1) % 2;
                let opponent = &self.players[opponent_index];

                if opponent.hand.len() == 0 {
                    return;
                }

                let opponent_ctx = AskContext {
                    actor: opponent_index,
                    source: ctx.source,
                    source_ability: ctx.source_ability,
                };

                loop {
                    let target_cards = agents[opponent_index].ask_cards_from_pile(
                        self,
                        &opponent_ctx,
                        PileFlag::HAND,
                        1,
                    );

                    if target_cards.len() != 1 {
                        continue;
                    }

                    let Some(target_card) = target_cards.get(0) else {
                        continue;
                    };

                    if target_card.0 != PileFlag::HAND {
                        continue;
                    }

                    let opponent = &mut self.players[opponent_index];

                    if target_card.1 >= opponent.hand.len() {
                        continue;
                    }

                    let card = opponent.hand.remove(target_card.1);
                    opponent.discard_pile.push(card);

                    break;
                }
            }
            Ability::CopyPlayedShip => {
                let eligible = self.players[actor]
                    .ships_in_play()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| (c.id != source).then_some(i))
                    .collect::<Vec<usize>>();

                if eligible.is_empty() {
                    return; // nonthing to do lol
                }

                loop {
                    let selected_ship_index = agents[actor].ask_played_ship(self, &ctx, &eligible);

                    if !eligible.contains(&selected_ship_index) {
                        continue;
                    }

                    let Some(&&selected_ship) =
                        self.players[actor].ships_in_play().get(selected_ship_index)
                    else {
                        continue;
                    };

                    let player = &mut self.players[actor];

                    player.in_play = player
                        .in_play
                        .iter()
                        .copied()
                        .map(|c| {
                            if c.id == source {
                                InPlayCard {
                                    id: c.id,
                                    name: c.name,
                                    copied_card: Some(selected_ship.name),
                                }
                            } else {
                                c
                            }
                        })
                        .collect();

                    player.faction_count[selected_ship.name.faction() as usize] += 1;
                    self.resolve_card_abilities(agents, actor, source, selected_ship.name);

                    break;
                }
            }
            Ability::NextAcquiredShipToTopOfDeck => {
                self.next_aquired_ship_to_top_of_deck = true;
            }
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
