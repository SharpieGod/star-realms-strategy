use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    marker::PhantomData,
    path::{self, Path, PathBuf},
    sync::LazyLock,
};

use rand::seq::{IndexedRandom, SliceRandom};
use serde::{Deserialize, Serialize};

use crate::{
    Faction::Unaligned,
    ResourceType::{Authority, Combat, Trade},
};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Amount {
    Number(u32),
    ShipsPlayed(Faction), // Neutral = any
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Faction {
    TradeFederation,
    Blob,
    StarEmpire,
    MachineCult,
    Unaligned,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Condition {
    HasAlly(Faction),
    And(Box<Condition>, Box<Condition>),
    BaseCountAtLeast(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum ResourceType {
    Authority,
    Combat,
    Trade,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum ScrapType {
    DiscardPile,
    Hand,
    HandOrDiscardPile,
}

#[derive(Serialize, Deserialize, Clone, Debug)]

enum Event {
    PlayShip(Faction), // Neutral = Any
    Scrap,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
enum CardType {
    Ship,
    Base { defense: u32, it_outpost: bool }, // defense: u32, is_outpost: bool
}

#[derive(Serialize, Deserialize, Debug)]
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
static CARDS: LazyLock<HashMap<CardNamed, Card>> = LazyLock::new(load_cards);

fn effect(name: &CardNamed) -> &'static Effect {
    &CARDS[name].effect
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

struct Player {
    personal_deck: Vec<Card>,
    hand: Vec<Card>,
    in_play: Vec<Card>,
    discard_pile: Vec<Card>,
    hp: u32,
}

struct Game<'a> {
    players: [&'a Player; 2],
    turn_number: u32,
}

enum PlayerAction {
    PlayCard(usize),
    Discard(usize),
    Card(usize, CardAction),
    DestroyTargetBase(usize),
}

enum ChoiceValue {
    Bool(bool),
    Index(usize),
    Indicies(Vec<usize>),
}
enum CardAction {
    Scrap,
    Choice(ChoiceValue),
}

fn main() {
    let card_counts = CardCounts::try_from(PathBuf::from("./cards/deck.ron")).unwrap();
    let mut deck: Vec<CardNamed> = card_counts.into();
    let mut rng = rand::rng();
    deck.shuffle(&mut rng);

    println!(
        "{:#?}",
        deck.iter()
            .take(5)
            .copied()
            .map(|n| &CARDS[&n])
            .collect::<Vec<&Card>>()
    );
}
