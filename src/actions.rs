use crate::effects::Effect;
use crate::player::InPlayCard;

#[derive(PartialEq, Eq)]
pub enum PlayerAction {
    PlayCard(usize),
    PlayAll,
    BuyCard(usize),
    Card(usize, CardAction),
    SpendCombat(CombatTarget),
    EndTurn,
}

#[derive(PartialEq, Eq)]
pub enum CombatTarget {
    Enemy,
    EnemyBase(usize),
}

#[derive(PartialEq, Eq)]
pub enum CardAction {
    Scrap,
    EngageEffect,
}

/// Who's being asked, and which card instance's effect is asking.
pub struct AskContext<'a> {
    pub actor: usize,
    pub source: InPlayCard,
    pub source_effect: &'a Effect,
}
