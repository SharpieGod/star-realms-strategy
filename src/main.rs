use std::{fs::OpenOptions, io::Write};

use serde::{Deserialize, Serialize};

use crate::{
    Faction::Neutral,
    ResourceType::{Attack, Authority, Trade},
};

#[derive(Serialize, Deserialize, Clone)]
enum Amount {
    Number(u32),
    ShipsPlayed(Faction), // Neutral = any
}

#[derive(Serialize, Deserialize, Clone)]
enum Faction {
    TradeFederation,
    Blob,
    StarEmpire,
    MachineCult,
    Neutral,
}

#[derive(Serialize, Deserialize, Clone)]
enum Condition {
    HasAlly(Faction),
    And(Box<Condition>, Box<Condition>),
    BaseCountAtLeast(u32),
}

#[derive(Serialize, Deserialize, Clone)]
enum ResourceType {
    Authority,
    Attack,
    Trade,
}

#[derive(Serialize, Deserialize, Clone)]
enum ScrapType {
    DiscardPile,
    Hand,
    HandOrDiscardPile,
}

#[derive(Serialize, Deserialize, Clone)]

enum Event {
    PlayShip(Faction), // Neutral = Any
}

#[derive(Serialize, Deserialize, Clone)]
enum Effect {
    If(Condition, Box<Effect>),
    Or(Box<Effect>, Box<Effect>),
    AndOr(Box<Effect>, Box<Effect>),
    Sequence(Vec<Effect>),
    On(Event, Box<Effect>),
    Draw(Amount),
    TargetedEffect(BaseEffect),
    May(Box<Effect>),
    /// Copy another ship played this turn; for ally checks the copy counts under
    /// both its own faction and the copied ship's faction.
    CopyPlayedShip,
    /// Sets a one-shot flag consumed when this player next acquires a ship this turn.
    NextAcquiredShipToTopOfDeck,
}

#[derive(Serialize, Deserialize, Clone)]
enum BaseEffect {
    DestroyTargetBase,
    ScrapCardInRow,
    OpponentDiscards,
    AquireShipForFree { to_top_of_deck: bool },
    Resource(ResourceType, u32),
    Scrap(ScrapType, u32),
    Discard(u32),
}

#[derive(Serialize, Deserialize)]
enum CardType {
    Ship,
    Base(u32, bool), // defense: u32, is_outpost: bool
}

#[derive(Serialize, Deserialize)]
struct Card {
    name: String,
    faction: Faction,
    card_type: CardType,
    cost: u32,
    effects: Vec<Effect>,
    on_scrap: Option<Effect>,
    /// Mech World: counts as an ally for every faction while in play.
    #[serde(default)]
    is_all_faction_ally: bool,
}

impl Card {
    fn save(&self) {
        let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(format!("./cards/{}.json", self.name))
        else {
            return;
        };

        file.write_all(serde_json::to_string_pretty(self).unwrap().as_bytes())
            .unwrap();
    }
}
trait State {}

struct Player {
    deck: Vec<Card>,
    discard_pile: Vec<Card>,
}

fn main() {}
