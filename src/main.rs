use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{self, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    Faction::Unaligned,
    ResourceType::{Authority, Combat, Trade},
};

#[derive(Serialize, Deserialize)]
enum CardNamed {
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
    Unaligned,
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
    Combat,
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
    Scrap,
}

#[derive(Serialize, Deserialize, Clone)]
enum Effect {
    // Logical Statements and conditions
    If(Condition, Box<Effect>),
    Or(Box<Effect>, Box<Effect>),
    AndOr(Box<Effect>, Box<Effect>),
    Sequence(Vec<Effect>),
    On(Event, Box<Effect>),
    Draw(Amount),
    May(Box<Effect>),

    // Actual Actions
    AquireShipForFree {
        to_top_of_deck: bool,
        max_cost: Option<u32>,
    },
    Resource(ResourceType, u32),
    Scrap(ScrapType, u32),
    Discard(u32),
    DestroyTargetBase,
    ScrapCardInRow,
    OpponentDiscards,
    /// Copy another ship played this turn; for ally checks the copy counts under
    /// both its own faction and the copied ship's faction.
    CopyPlayedShip,
    /// Sets a one-shot flag consumed when this player next acquires a ship this turn.
    NextAcquiredShipToTopOfDeck,
}

#[derive(Serialize, Deserialize)]
enum CardType {
    Ship,
    Base { defense: u32, it_outpost: bool }, // defense: u32, is_outpost: bool
}

#[derive(Serialize, Deserialize)]
struct Card {
    name: CardNamed,
    faction: Faction,
    card_type: CardType,
    cost: u32,
    effect: Effect,
    /// Mech World: counts as an ally for every faction while in play.
    #[serde(default)]
    is_all_faction_ally: bool,
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

/// A deck manifest: card name paired with how many copies it contains.
#[derive(Serialize, Deserialize)]
struct Deck(Vec<(CardNamed, u32)>);

impl TryFrom<PathBuf> for Deck {
    type Error = ();
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let Ok(s) = fs::read_to_string(path) else {
            return Err(());
        };

        ron::de::from_str::<Self>(s.as_str()).map_err(|_| ())
    }
}
trait State {}

struct Player {
    personal_deck: Vec<Card>,
    hand: Vec<Card>,
    on_board: Vec<Card>,
    discard_pile: Vec<Card>,
    hp: u32,
}

fn main() {}
