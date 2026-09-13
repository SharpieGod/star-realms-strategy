use std::{collections::HashMap, fmt::Display};

use rand::{rngs::ThreadRng, seq::SliceRandom};

use crate::abilities::Ability;
use crate::cards::{CARDS, CardNamed, CardType, STARTER_PERSONAL_DECK};
use crate::faction::Faction;

/// Identifies one specific card instance in play, distinct from others of the
/// same `CardNamed` (e.g. two Vipers), independent of its position in `in_play`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CardInstanceId(pub usize);

#[derive(Clone, Copy)]
pub struct InPlayCard {
    pub id: CardInstanceId,
    pub name: CardNamed,
}

pub struct Player {
    pub personal_deck: Vec<CardNamed>,
    pub hand: Vec<CardNamed>,
    pub in_play: Vec<InPlayCard>,
    pub discard_pile: Vec<CardNamed>,
    pub authority: u32,
    pub trade: u32,
    pub combat: u32,
    pub pending_ally_abilities: Vec<(InPlayCard, Faction, Ability)>,
    pub next_instance_id: usize,
}

impl Player {
    pub fn play_card(&mut self, name: CardNamed) -> CardInstanceId {
        let id = CardInstanceId(self.next_instance_id);
        self.next_instance_id += 1;
        self.in_play.push(InPlayCard { id, name });
        id
    }

    pub fn faction_count(&self) -> HashMap<Faction, usize> {
        let mut map = HashMap::new();
        for i in &self.in_play {
            *map.entry(i.name.faction()).or_default() += 1;
        }

        map
    }

    /// Removes one card instance from play (scrapped, discarded, destroyed),
    /// dropping any of its still-unused pending Ally abilities along with it.
    pub fn remove_from_play(&mut self, id: CardInstanceId) -> Option<CardNamed> {
        let pos = self.in_play.iter().position(|c| c.id == id)?;
        let card = self.in_play.remove(pos);
        self.pending_ally_abilities
            .retain(|source| source.0.id != id);
        Some(card.name)
    }

    pub fn get_card_instace(&self, id: CardInstanceId) -> Option<CardNamed> {
        self.in_play.iter().find(|c| c.id == id).map(|c| c.name)
    }

    pub fn draw_cards(&mut self, mut n: usize, rng: &mut ThreadRng) {
        while n > 0 && !(self.personal_deck.is_empty() && self.discard_pile.is_empty()) {
            while n > 0
                && let Some(card) = self.personal_deck.pop()
            {
                self.hand.push(card);
                n -= 1;
            }

            if n > 0 {
                self.personal_deck = self.discard_pile.drain(..).collect();
                self.personal_deck.shuffle(rng);
            }
        }
    }

    pub fn outposts_in_play(&self) -> Vec<InPlayCard> {
        self.bases_in_play()
            .into_iter()
            .filter(|b| {
                matches!(
                    CARDS[&b.name].card_type,
                    CardType::Base {
                        defense: _,
                        is_outpost: true
                    }
                )
            })
            .collect()
    }

    pub fn bases_in_play(&self) -> Vec<InPlayCard> {
        self.in_play
            .iter()
            .copied()
            .filter(|c| CARDS[&c.name].is_base())
            .collect()
    }

    pub fn ships_in_play(&self) -> Vec<InPlayCard> {
        self.in_play
            .iter()
            .copied()
            .filter(|c| !CARDS[&c.name].is_base())
            .collect()
    }
}

impl Default for Player {
    fn default() -> Self {
        let mut deck = STARTER_PERSONAL_DECK.clone();
        let mut rng = rand::rng();
        deck.shuffle(&mut rng);

        #[cfg(feature = "reset_resources")]
        let (trade, combat) = (0, 0);
        #[cfg(not(feature = "reset_resources"))]
        let (trade, combat) = (100, 100); // only for testing

        Self {
            personal_deck: deck,
            hand: Default::default(),
            in_play: Default::default(),
            discard_pile: Default::default(),
            authority: 50,
            trade,
            combat,
            pending_ally_abilities: Default::default(),
            next_instance_id: 0,
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "authority: {}, combat: {}, trade: {}",
            self.authority, self.combat, self.trade
        )?;
        writeln!(f, "deck: {} cards", self.personal_deck.len())?;
        writeln!(
            f,
            "bases in play: {}",
            self.in_play
                .iter()
                .filter(|c| CARDS[&c.name].is_base())
                .map(|c| c.name.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        writeln!(
            f,
            "ships in play: {}",
            self.in_play
                .iter()
                .filter(|c| !CARDS[&c.name].is_base())
                .map(|c| c.name.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        writeln!(
            f,
            "hand: {}",
            self.hand
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        write!(
            f,
            "discard pile: {}",
            self.discard_pile
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
