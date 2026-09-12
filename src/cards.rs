use std::{collections::HashMap, fmt::Display, fs, path::PathBuf, sync::LazyLock};

use colorize::AnsiColor;
use serde::{Deserialize, Serialize};

use crate::effects::Effect;
use crate::faction::Faction::{self, Unaligned};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CardNamed {
    BarterWorld,
    BattleBlob,
    BattleMech,
    BattlePod,
    BattleStation,
    Battlecruiser,
    BlobCarrier,
    BlobDestroyer,
    BlobFighter,
    BlobWheel,
    BlobWorld,
    BrainWorld,
    CentralOffice,
    CommandShip,
    Corvette,
    Cutter,
    DefenseCenter,
    Dreadnaught,
    EmbassyYacht,
    Explorer,
    FederationShuttle,
    Flagship,
    FleetHQ,
    Freighter,
    ImperialFighter,
    ImperialFrigate,
    Junkyard,
    MachineBase,
    MechWorld,
    MissileBot,
    MissileMech,
    Mothership,
    PatrolMech,
    PortofCall,
    Ram,
    RecyclingStation,
    RoyalRedoubt,
    Scout,
    SpaceStation,
    StealthNeedle,
    SupplyBot,
    SurveyShip,
    TheHive,
    TradeBot,
    TradeEscort,
    TradePod,
    TradingPost,
    Viper,
    WarWorld,
}

impl Display for CardNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = match self {
            CardNamed::BarterWorld => "Barter World",
            CardNamed::BattleBlob => "Battle Blob",
            CardNamed::BattleMech => "Battle Mech",
            CardNamed::BattlePod => "Battle Pod",
            CardNamed::BattleStation => "Battle Station",
            CardNamed::Battlecruiser => "Battlecruiser",
            CardNamed::BlobCarrier => "Blob Carrier",
            CardNamed::BlobDestroyer => "Blob Destroyer",
            CardNamed::BlobFighter => "Blob Fighter",
            CardNamed::BlobWheel => "Blob Wheel",
            CardNamed::BlobWorld => "Blob World",
            CardNamed::BrainWorld => "Brain World",
            CardNamed::CentralOffice => "Central Office",
            CardNamed::CommandShip => "Command Ship",
            CardNamed::Corvette => "Corvette",
            CardNamed::Cutter => "Cutter",
            CardNamed::DefenseCenter => "Defense Center",
            CardNamed::Dreadnaught => "Dreadnaught",
            CardNamed::EmbassyYacht => "Embassy Yacht",
            CardNamed::Explorer => "Explorer",
            CardNamed::FederationShuttle => "Federation Shuttle",
            CardNamed::Flagship => "Flagship",
            CardNamed::FleetHQ => "Fleet HQ",
            CardNamed::Freighter => "Freighter",
            CardNamed::ImperialFighter => "Imperial Fighter",
            CardNamed::ImperialFrigate => "Imperial Frigate",
            CardNamed::Junkyard => "Junkyard",
            CardNamed::MachineBase => "Machine Base",
            CardNamed::MechWorld => "Mech World",
            CardNamed::MissileBot => "Missile Bot",
            CardNamed::MissileMech => "Missile Mech",
            CardNamed::Mothership => "Mothership",
            CardNamed::PatrolMech => "Patrol Mech",
            CardNamed::PortofCall => "Port of Call",
            CardNamed::Ram => "Ram",
            CardNamed::RecyclingStation => "Recycling Station",
            CardNamed::RoyalRedoubt => "Royal Redoubt",
            CardNamed::Scout => "Scout",
            CardNamed::SpaceStation => "Space Station",
            CardNamed::StealthNeedle => "Stealth Needle",
            CardNamed::SupplyBot => "Supply Bot",
            CardNamed::SurveyShip => "Survey Ship",
            CardNamed::TheHive => "The Hive",
            CardNamed::TradeBot => "Trade Bot",
            CardNamed::TradeEscort => "Trade Escort",
            CardNamed::TradePod => "Trade Pod",
            CardNamed::TradingPost => "Trading Post",
            CardNamed::Viper => "Viper",
            CardNamed::WarWorld => "War World",
        };

        write!(
            f,
            "{}",
            match CARDS[self].faction {
                Faction::TradeFederation => n.blue(),
                Faction::Blob => n.green(),
                Faction::StarEmpire => n.yellow(),
                Faction::MachineCult => n.red(),
                Unaligned => n.grey(),
            }
        )
    }
}

impl CardNamed {
    pub fn faction(&self) -> Faction {
        CARDS[self].faction
    }

    pub fn with_cost(&self) -> String {
        format!("{self} ({})", CARDS[self].cost.to_string().b_yellow())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CardType {
    Ship,
    Base { defense: u32, is_outpost: bool },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub name: CardNamed,
    pub faction: Faction,
    pub card_type: CardType,
    pub cost: u32,
    pub effects: Vec<Effect>,
    /// Mech World: counts as an ally for every faction while in play.
    #[serde(default)]
    pub is_all_faction_ally: bool,
}

impl From<CardNamed> for &Card {
    fn from(value: CardNamed) -> Self {
        &CARDS[&value]
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}){}\n{}",
            self.name,
            self.cost.to_string().b_yellow(),
            match self.card_type {
                CardType::Ship => "".to_string(),
                CardType::Base {
                    defense,
                    is_outpost,
                } => format!(
                    " {defense} defense{}",
                    if is_outpost { " outpost" } else { "" }
                ),
            },
            if self.effects.is_empty() {
                "no effects".to_string()
            } else {
                self.effects
                    .iter()
                    .map(|e| format!("\t{e}"))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        )?;

        Ok(())
    }
}

impl Card {
    pub fn name_only(&self) -> String {
        if self.cost > 0 {
            format!("({}) {}", self.cost.to_string().b_yellow(), self.name)
        } else {
            self.name.to_string()
        }
    }

    pub fn is_base(&self) -> bool {
        match self.card_type {
            CardType::Ship => false,
            CardType::Base {
                defense: _,
                is_outpost: _,
            } => true,
        }
    }

    pub fn is_outpost(&self) -> bool {
        matches!(
            self.card_type,
            CardType::Base {
                defense: _,
                is_outpost: true
            }
        )
    }

    pub fn get_base_defense(&self) -> Option<u32> {
        match self.card_type {
            CardType::Ship => None,
            CardType::Base {
                defense,
                is_outpost: _,
            } => Some(defense),
        }
    }
}

impl TryFrom<PathBuf> for Card {
    type Error = ();
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let Ok(s) = fs::read_to_string(path) else {
            return Err(());
        };

        ron::de::from_str::<Self>(s.as_str()).map_err(|_| ())
    }
}

/// Directories scanned at startup to populate `CARDS`.
const CARD_DIRS: &[&str] = &["cards/core-set"];

fn load_cards() -> HashMap<CardNamed, Card> {
    let mut cards = HashMap::new();
    for dir in CARD_DIRS {
        for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read {dir}: {e}")) {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("ron") {
                continue;
            }
            let card = Card::try_from(path.clone())
                .unwrap_or_else(|_| panic!("failed to parse card at {}", path.display()));
            cards.insert(card.name, card);
        }
    }
    cards
}

/// All cards, keyed by name, loaded once on first access.
pub static CARDS: LazyLock<HashMap<CardNamed, Card>> = LazyLock::new(load_cards);
pub static STARTER_PERSONAL_DECK: LazyLock<Vec<CardNamed>> = LazyLock::new(|| {
    #[cfg(feature = "reset_resources")]
    let counts = vec![(CardNamed::Viper, 2), (CardNamed::Scout, 8)];
    #[cfg(not(feature = "reset_resources"))]
    let counts = vec![(CardNamed::BlobCarrier, 5), (CardNamed::Scout, 1)];

    Vec::<CardNamed>::from(CardCounts(counts))
});

pub static STARTER_GAME_DECK: LazyLock<Vec<CardNamed>> = LazyLock::new(|| {
    #[cfg(feature = "reset_resources")]
    let path_str = "./cards/deck.ron";
    #[cfg(not(feature = "reset_resources"))]
    let path_str = "./cards/deck.ron.dev";

    let card_counts = CardCounts::try_from(PathBuf::from(path_str)).unwrap();
    card_counts.into()
});

pub fn effect(name: &CardNamed) -> &'static Vec<Effect> {
    &CARDS[name].effects
}

/// A deck manifest: card name paired with how many copies it contains.
#[derive(Serialize, Deserialize)]
struct CardCounts(Vec<(CardNamed, u8)>);

impl TryFrom<PathBuf> for CardCounts {
    type Error = ();
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let Ok(s) = fs::read_to_string(path) else {
            return Err(());
        };

        ron::de::from_str::<Self>(s.as_str()).map_err(|_| ())
    }
}

impl From<CardCounts> for Vec<CardNamed> {
    fn from(counts: CardCounts) -> Self {
        counts
            .0
            .iter()
            .copied()
            .flat_map(|(c, count)| vec![c; count as usize])
            .collect::<Vec<CardNamed>>()
    }
}
