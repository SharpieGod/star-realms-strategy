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
    And(Box<Condition>, Box<Condition>),
    BaseCountAtLeast(u32),
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
    /// Ally ability: usable at will during your main phase, once you have another
    /// card of this faction in play. Usable once per play of this card.
    Ally(Faction, Box<Effect>),
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
            Effect::Ally(faction, effect) => write!(f, "{faction} ally: {{{effect}}}"),
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
                Unaligned => n.grey(),
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

        if self.cost > 0 {
            format!(
                "({}) {}",
                self.cost.to_string().b_yellow(),
                match self.faction {
                    Faction::TradeFederation => n.blue(),
                    Faction::Blob => n.green(),
                    Faction::StarEmpire => n.yellow(),
                    Faction::MachineCult => n.red(),
                    Unaligned => n.grey(),
                }
            )
        } else {
            match self.faction {
                Faction::TradeFederation => n.blue(),
                Faction::Blob => n.green(),
                Faction::StarEmpire => n.yellow(),
                Faction::MachineCult => n.red(),
                Unaligned => n.grey(),
            }
        }
    }

    fn is_base(&self) -> bool {
        match self.card_type {
            CardType::Ship => false,
            CardType::Base {
                defense: _,
                is_outpost: _,
            } => true,
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
    pending_ally_effects: Vec<(CardNamed, Effect)>,
}

impl Player {
    fn draw_cards(&mut self, mut n: usize, rng: &mut ThreadRng) {
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
}

impl Default for Player {
    fn default() -> Self {
        let mut deck = STARTER_PERSONAL_DECK.clone();
        let mut rng = rand::rng();
        deck.shuffle(&mut rng);

        Self {
            personal_deck: deck,
            hand: Default::default(),
            in_play: Default::default(),
            discard_pile: Default::default(),
            authority: 50,
            trade: Default::default(),
            combat: Default::default(),
            pending_ally_effects: Default::default(),
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
            "hand: {}",
            self.hand
                .iter()
                .map(|c| CARDS[c].name_only())
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        writeln!(
            f,
            "in play: {}",
            self.in_play
                .iter()
                .map(|c| CARDS[c].name_only())
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        write!(
            f,
            "discard pile: {}",
            self.discard_pile
                .iter()
                .map(|c| CARDS[c].name_only())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "player 1:")?;
        writeln!(f, "{}", self.players[0])?;
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
        writeln!(f, "player 2:")?;
        writeln!(f, "{}", self.players[1])
    }
}

impl Game {
    fn new() -> Game {
        let mut player1 = Player::default();
        let mut player2 = Player::default();
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
            deck,
            shop: *shop.as_array().unwrap(),
        }
    }

    fn do_action(&mut self, actor: usize, action: PlayerAction) {
        let player = &mut self.players[actor];

        match action {
            PlayCard(hand_index) => {
                let played_card = player.hand.remove(hand_index);
                player.in_play.push(played_card);

                self.resolve_effect(
                    actor,
                    played_card,
                    &Effect::Sequence(CARDS[&played_card].effects.clone()),
                );
            }
            PlayerAction::BuyCard(card_index) => {
                let Some(target_card) = self.shop[card_index] else {
                    return;
                };

                self.shop[card_index] = self.deck.pop(); // Draw new card, if no card its None anyways

                player.discard_pile.push(target_card); // TODO: next card from shop to top of deck
            }
            PlayerAction::Card(card_index, card_action) => {
                let Some(&target_card) = player.in_play.get(card_index) else {
                    return;
                };

                match card_action {
                    CardAction::Scrap => {
                        let Some(Effect::ScrapAbility(inner)) = CARDS[&target_card]
                            .effects
                            .iter()
                            .find(|e| matches!(e, Effect::ScrapAbility(_)))
                        else {
                            return;
                        };

                        player.in_play.remove(card_index);
                        self.resolve_effect(actor, target_card, inner);
                    }
                    CardAction::Choice(_) => todo!(),
                }
            }
            PlayerAction::DestroyTargetBase(_) => todo!(),
            PlayerAction::SpendCombat(combat_target) => todo!(),
            PlayerAction::EndTurn => todo!(),
        }
    }

    fn resolve_effect(&mut self, actor: usize, source: CardNamed, effect: &Effect) {
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
                    self.resolve_effect(actor, source, e);
                }
            }
            Effect::If(condition, inner) => {
                if self.evaluate_condition(actor, condition) {
                    self.resolve_effect(actor, source, inner);
                }
            }
            // Ally abilities become available for at-will use once queued here;
            // they aren't resolved immediately like a normal effect.
            Effect::Ally(_, inner) => {
                self.players[actor]
                    .pending_ally_effects
                    .push((source, (**inner).clone()));
            }
            // Held abilities: never walked during normal play, only looked up
            // on demand (ScrapInPlay action, or the PlayShip trigger scan) —
            // so they're intentional no-ops here.
            Effect::ScrapAbility(_) | Effect::Trigger(_, _) => {}
            _ => todo!(),
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
                    .filter(|c| CARDS[c].is_base())
                    .count()
                    >= *count as usize
            }
        }
    }
}

enum PlayerAction {
    PlayCard(usize), // In play always
    BuyCard(usize),
    Card(usize, CardAction),
    DestroyTargetBase(usize),
    SpendCombat(CombatTarget),
    EndTurn,
}

enum CombatTarget {
    Enemy,
    EnemyBase(usize),
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

enum ChoiceKind {
    YesNo,                                          // May, each half of AndOr, Or's branch pick
    SelectFromPile { pile: ScrapType, count: u32 }, // Scrap, Discard (pile: Hand), OpponentDiscards (pile: Hand)
    SelectShopCard { eligible: Vec<usize> }, // AquireShipForFree (after a YesNo), ScrapCardInRow
    SelectEnemyBase { eligible: Vec<usize> }, // DestroyTargetBase
    SelectPlayedShip { eligible: Vec<CardNamed> }, // CopyPlayedShip
}

enum GamePhase {
    Main,
    Drawing,
}

trait Agent {
    fn choose_action(&mut self, game: &Game, actor: usize) -> PlayerAction;
    fn choose(&mut self, game: &Game, actor: usize, kind: &ChoiceKind) -> ChoiceValue;
}

fn main() {
    let mut game = Game::new();

    println!("{}", game);
}
