use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, OpenOptions},
    io::Write,
    marker::PhantomData,
    path::{self, Path, PathBuf},
    sync::LazyLock,
};

use colorize::AnsiColor;
use rand::{
    RngExt,
    rngs::ThreadRng,
    seq::{IndexedRandom, SliceRandom},
};
use serde::{Deserialize, Serialize};

use crate::{
    CardIndex::InPlay,
    CardNamed::{Scout, Viper},
    Faction::Unaligned,
    PlayerAction::PlayCard,
    Resource::{Authority, Combat, Trade},
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

impl Display for CardNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
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
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Amount {
    Number(u32),
    ShipsPlayed(Faction), // Neutral = any
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Amount::Number(n) => write!(f, "{n}"),
            Amount::ShipsPlayed(faction) => write!(f, "# of {faction} played"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
enum Faction {
    TradeFederation,
    Blob,
    StarEmpire,
    MachineCult,
    Unaligned,
}

impl Display for Faction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{:?}", self);

        write!(
            f,
            "{}",
            match self {
                Faction::TradeFederation => s.blue(),
                Faction::Blob => s.green(),
                Faction::StarEmpire => s.yellow(),
                Faction::MachineCult => s.red(),
                Unaligned => s,
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Condition {
    HasAlly(Faction),
    And(Box<Condition>, Box<Condition>),
    BaseCountAtLeast(u32),
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::HasAlly(faction) => write!(f, "{faction} Ally"),
            Condition::And(condition, condition1) => write!(f, "{condition} and {condition1}"),
            Condition::BaseCountAtLeast(count) => write!(f, "at least {count} bases"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Resource {
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
enum ScrapType {
    DiscardPile,
    Hand,
    HandOrDiscardPile,
}

impl Display for ScrapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ScrapType::DiscardPile => "Discard Pile",
                ScrapType::Hand => "Hand",
                ScrapType::HandOrDiscardPile => "Hand or Discard Pile",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]

enum Event {
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
enum Effect {
    // Logical Statements and conditions
    If(Condition, Box<Effect>),
    Or(Box<Effect>, Box<Effect>),
    AndOr(Box<Effect>, Box<Effect>),
    Sequence(Vec<Effect>),
    /// "Scrap this: ...". Available at will while the card sits in play.
    ScrapAbility(Box<Effect>),
    /// "Whenever <event>, ...". Fires reactively while the card is in play.
    Trigger(Event, Box<Effect>),
    Draw(Amount),
    May(Box<Effect>),

    // Actual Actions
    AquireShipForFree {
        to_top_of_deck: bool,
        max_cost: Option<u32>,
    },
    Resource(Resource, u32),
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

impl Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::If(condition, effect) => write!(f, "{{{condition}}}: {{{effect}}}"),
            Effect::Or(effect, effect1) => write!(f, "{{{effect}}} or {{{effect1}}}"),
            Effect::AndOr(effect, effect1) => write!(f, "{{{effect}}} and/or {{{effect1}}}"),
            Effect::Sequence(effects) => write!(
                f,
                "{}",
                effects
                    .iter()
                    .map(|e| format!("{{{e}}}"))
                    .collect::<Vec<String>>()
                    .join(" -> ")
            ),
            Effect::ScrapAbility(effect) => write!(f, "scrap this: {{{effect}}}"),
            Effect::Trigger(event, effect) => write!(f, "whenever {event}: {{{effect}}}"),
            Effect::Draw(amount) => write!(
                f,
                "Draw {}",
                match amount {
                    Amount::Number(count) => format!("{}", n_cards(*count)),
                    Amount::ShipsPlayed(_) => amount.to_string(),
                }
            ),
            Effect::May(effect) => write!(f, "may {{{effect}}}"),
            Effect::AquireShipForFree {
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
            Effect::Resource(resource_type, count) => write!(f, "Gain {count} {resource_type}"),
            Effect::Scrap(scrap_type, count) => {
                write!(f, "scrap {} in {scrap_type}", n_cards(*count))
            }
            Effect::Discard(count) => write!(f, "Discard {}", n_cards(*count)),
            Effect::DestroyTargetBase => write!(f, "destroy target base"),
            Effect::ScrapCardInRow => write!(f, "scrap card in trade row"),
            Effect::OpponentDiscards => write!(f, "opponent discards a card"),
            Effect::CopyPlayedShip => write!(f, "copy played ship"),
            Effect::NextAcquiredShipToTopOfDeck => write!(f, "next aquired ship on top of deck"),
        }
    }
}

fn n_cards(n: u32) -> String {
    format!("{n} {}", if n == 1 { "card" } else { "cards" })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum CardType {
    Ship,
    Base { defense: u32, is_outpost: bool },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Card {
    name: CardNamed,
    faction: Faction,
    card_type: CardType,
    cost: u32,
    effects: Vec<Effect>,
    /// Mech World: counts as an ally for every faction while in play.
    #[serde(default)]
    is_all_faction_ally: bool,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.name.to_string();

        write!(
            f,
            "({}) {}{}\n{}",
            self.cost.to_string().b_yellow(),
            match self.faction {
                Faction::TradeFederation => n.blue(),
                Faction::Blob => n.green(),
                Faction::StarEmpire => n.yellow(),
                Faction::MachineCult => n.red(),
                Unaligned => n,
            },
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
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        )?;

        Ok(())
    }
}

impl Card {
    fn name_only(&self) -> String {
        let n = self.name.to_string();

        match self.faction {
            Faction::TradeFederation => n.blue(),
            Faction::Blob => n.green(),
            Faction::StarEmpire => n.yellow(),
            Faction::MachineCult => n.red(),
            Unaligned => n,
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
static CARDS: LazyLock<HashMap<CardNamed, Card>> = LazyLock::new(load_cards);
static STARTER_PERSONAL_DECK: LazyLock<Vec<CardNamed>> =
    LazyLock::new(|| Vec::<CardNamed>::from(CardCounts(vec![(Viper, 2), (Scout, 8)])));
static STARTER_GAME_DECK: LazyLock<Vec<CardNamed>> = LazyLock::new(|| {
    let card_counts = CardCounts::try_from(PathBuf::from("./cards/deck.ron")).unwrap();
    card_counts.into()
});
fn effect(name: &CardNamed) -> &'static Vec<Effect> {
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

struct Player {
    personal_deck: Vec<CardNamed>,
    hand: Vec<CardNamed>,
    in_play: Vec<CardNamed>,
    discard_pile: Vec<CardNamed>,
    authority: u32,
    trade: u32,
    combat: u32,
}

impl Default for Player {
    fn default() -> Self {
        let starter_deck = Vec::<CardNamed>::from(CardCounts(vec![(Viper, 2), (Scout, 8)]));
        Self {
            personal_deck: STARTER_PERSONAL_DECK.clone(),
            hand: Default::default(),
            in_play: Default::default(),
            discard_pile: Default::default(),
            authority: 50,
            trade: Default::default(),
            combat: Default::default(),
        }
    }
}

struct Game {
    players: [Player; 2],
    turn_number: u32,
    rng: ThreadRng,
    deck: Vec<CardNamed>,
    shop: [Option<CardNamed>; 5],
}

impl Game {
    fn new() -> Game {
        let player1 = Player::default();
        let player2 = Player::default();
        let mut rng = rand::rng();

        let turn_number = rng.random_range(0..=1);
        let mut deck = STARTER_GAME_DECK.clone();
        deck.shuffle(&mut rng);

        let shop = deck
            .iter()
            .copied()
            .take(5)
            .map(|c| Some(c))
            .collect::<Vec<Option<CardNamed>>>();

        Self {
            players: [player1, player2],
            turn_number,
            rng,
            deck,
            shop: *shop.as_array().unwrap(),
        }
    }

    fn do_action(&mut self, acting_player: usize, action: PlayerAction) {
        let player = &mut self.players[acting_player];

        match action {
            PlayCard(hand_index) => {
                let card_played = player.hand.remove(hand_index);
                player.in_play.push(card_played);

                for e in &CARDS[&card_played].effects {
                    match e {
                        Effect::Resource(resource, count) => match resource {
                            Authority => player.authority += count,
                            Combat => player.combat += count,
                            Trade => player.trade += count,
                        },
                        _ => {}
                    }
                }
            }
            PlayerAction::BuyCard(_) => todo!(),
            PlayerAction::Card(card_index, card_action) => todo!(),
            PlayerAction::DestroyTargetBase(_) => todo!(),
            PlayerAction::SpendCombat(combat_target) => todo!(),
            PlayerAction::EndTurn => todo!(),
        }
    }
}

enum PlayerAction {
    PlayCard(usize), // In play always
    BuyCard(usize),
    Card(CardIndex, CardAction),
    DestroyTargetBase(usize),
    SpendCombat(CombatTarget),
    EndTurn,
}

enum CombatTarget {
    Enemy,
    EnemyBase(usize),
}

enum CardIndex {
    InPlay(usize),
    DiscardPile(usize), // for selecting scraps
}
enum ChoiceValue {
    Bool(bool),
    Index(CardIndex),
    Indicies(Vec<CardIndex>),
}

enum CardAction {
    Scrap,
    Choice(ChoiceValue),
}

fn main() {
    let mut game = Game::new();

    println!(
        "{}",
        game.shop
            .iter()
            .enumerate()
            .map(|(i, c)| format!("({i}) {}", (&CARDS[&c.unwrap()]).name_only()))
            .collect::<Vec<String>>()
            .join("\n")
    );
}
