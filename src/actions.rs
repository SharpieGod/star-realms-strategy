use crate::abilities::Ability;
use crate::player::{CardInstanceId, InPlayCard};

#[derive(PartialEq, Eq)]
pub enum PlayerAction {
    PlayCard(usize),
    PlayAll,
    BuyCard(usize),
    Scrap { is_base: bool, index: usize },
    SpendCombat(CombatTarget),
    EndTurn,
}

#[derive(PartialEq, Eq)]
pub enum CombatTarget {
    Enemy,
    EnemyBase(usize),
}

/// Who's being asked, and which card instance's ability is asking.
pub struct AskContext<'a> {
    pub actor: usize,
    pub source: CardInstanceId,
    pub source_ability: &'a Ability,
}
