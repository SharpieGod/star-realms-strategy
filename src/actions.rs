use crate::effects::Effect;
use crate::player::InPlayCard;

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

/// Who's being asked, and which card instance's effect is asking.
pub struct AskContext<'a> {
    pub actor: usize,
    pub source: InPlayCard,
    pub source_effect: &'a Effect,
}
