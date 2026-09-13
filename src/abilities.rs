use std::fmt::Display;

use bitflags::bitflags;
use colorize::AnsiColor;
use serde::{Deserialize, Serialize};

use crate::faction::Faction;
use crate::game::Game;

use Resource::{Authority, Combat, Trade};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Amount {
    Number(usize),
    ShipsPlayed(Faction), // Neutral = any
}

impl Amount {
    pub fn compute(&self, game: &Game) -> usize {
        match self {
            Amount::Number(n) => *n,
            Amount::ShipsPlayed(faction) => {
                let actor = game.turn_number as usize % 2;
                let player = &game.players[actor];

                player
                    .in_play
                    .iter()
                    .filter_map(|c| (c.name.faction() == *faction).then_some(1))
                    .sum()
            }
        }
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Amount::Number(n) => write!(f, "{n}"),
            Amount::ShipsPlayed(faction) => write!(f, "# of {faction} played"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Condition {
    And(Box<Condition>, Box<Condition>),
    BaseCountAtLeast(usize),
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::And(condition, condition1) => write!(f, "{condition} and {condition1}"),
            Condition::BaseCountAtLeast(count) => write!(f, "at least {count} bases"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Resource {
    Authority,
    Combat,
    Trade,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{self:?}");

        write!(
            f,
            "{}",
            match self {
                Authority => s.green(),
                Combat => s.red(),
                Trade => s.yellow(),
            },
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]

pub enum Event {
    PlayShip(Faction), // Neutral = Any
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::PlayShip(faction) => write!(
                f,
                "{} played",
                if *faction != Faction::Unaligned {
                    faction.to_string()
                } else {
                    "any".to_string()
                }
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Ability {
    // Logical Statements and conditions
    If(Condition, Box<Ability>),
    Or(Box<Ability>, Box<Ability>),
    Sequence(Vec<Ability>),
    /// "Scrap this: ...". Available at will while the card sits in play.
    ScrapAbility(Box<Ability>),
    /// "Whenever <event>, ...". Fires reactively while the card is in play.
    Trigger(Event, Box<Ability>),
    /// Ally ability: usable at will during your main phase, once you have another
    /// card of this faction in play. Usable once per play of this card.
    Ally(Faction, Box<Ability>),
    Draw(Amount),
    May(Box<Ability>),

    // Actual Actions
    AquireShipForFree {
        to_top_of_deck: bool,
        max_cost: Option<u32>,
    },
    Resource(Resource, u32),
    Scrap(PileFlag, usize),
    Discard(usize),
    DestroyTargetBase,
    ScrapCardInRow,
    OpponentDiscards,
    /// Copy another ship played this turn; for ally checks the copy counts under
    /// both its own faction and the copied ship's faction.
    CopyPlayedShip,
    /// Sets a one-shot flag consumed when this player next acquires a ship this turn.
    NextAcquiredShipToTopOfDeck,
}

impl Display for Ability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ability::If(condition, ability) => write!(f, "{{{condition}}}: {{{ability}}}"),
            Ability::Or(ability, ability1) => write!(f, "{{{ability}}} or {{{ability1}}}"),
            Ability::Sequence(abilities) => match abilities.as_slice() {
                [Ability::May(ability), Ability::May(ability1)] => {
                    write!(f, "{{{ability}}} and/or {{{ability1}}}")
                }
                _ => write!(
                    f,
                    "{}",
                    abilities
                        .iter()
                        .map(|e| format!("{{{e}}}"))
                        .collect::<Vec<String>>()
                        .join(" -> ")
                ),
            },
            Ability::ScrapAbility(ability) => write!(f, "Scrap: {{{ability}}}"),
            Ability::Trigger(event, ability) => write!(f, "whenever {event}: {{{ability}}}"),
            Ability::Ally(faction, ability) => write!(f, "{{{faction} ally}}: {{{ability}}}"),
            Ability::Draw(amount) => write!(
                f,
                "Draw {}",
                match amount {
                    Amount::Number(count) => format!("{}", n_cards(*count)),
                    Amount::ShipsPlayed(_) => amount.to_string(),
                }
            ),
            Ability::May(ability) => write!(f, "may {{{ability}}}"),
            Ability::AquireShipForFree {
                to_top_of_deck,
                max_cost,
            } => write!(
                f,
                "aquire ship for free{}{}",
                if *to_top_of_deck {
                    " to top of deck"
                } else {
                    ""
                },
                if let Some(cost) = max_cost {
                    format!(" less than {cost}")
                } else {
                    "".to_string()
                }
            ),
            Ability::Resource(resource_type, count) => write!(f, "Gain {count} {resource_type}"),
            Ability::Scrap(pile, count) => {
                write!(f, "scrap {} in {pile}", n_cards(*count))
            }
            Ability::Discard(count) => write!(f, "Discard {}", n_cards(*count)),
            Ability::DestroyTargetBase => write!(f, "destroy target base"),
            Ability::ScrapCardInRow => write!(f, "scrap card in trade row"),
            Ability::OpponentDiscards => write!(f, "opponent discards a card"),
            Ability::CopyPlayedShip => write!(f, "copy played ship"),
            Ability::NextAcquiredShipToTopOfDeck => write!(f, "next aquired ship on top of deck"),
        }
    }
}

fn n_cards(n: usize) -> String {
    format!("{n} {}", if n == 1 { "card" } else { "cards" })
}

bitflags! {
    // `transparent` forwards to the inner flags type, whose Serialize/Deserialize
    // (from bitflags' `serde` feature, Cargo.toml) read/write names like
    // "HAND | DISCARD_PILE" in .ron instead of the raw bits.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
    #[serde(transparent)]
    pub struct PileFlag: u8 {
        const HAND = 0b01;
        const DISCARD_PILE = 0b10;
    }
}

impl Display for PileFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = if self.contains(PileFlag::HAND) && self.contains(PileFlag::DISCARD_PILE) {
            "Hand or Discard Pile"
        } else if self.contains(PileFlag::HAND) {
            "Hand"
        } else if self.contains(PileFlag::DISCARD_PILE) {
            "Discard Pile"
        } else {
            "no pile"
        };
        write!(f, "{s}")
    }
}
