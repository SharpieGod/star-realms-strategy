use crate::actions::{AskContext, PlayerAction};
use crate::cards::CardNamed;
use crate::effects::PileFlag;
use crate::game::Game;

pub trait Agent {
    fn choose_action(&mut self, game: &Game, actor: usize) -> PlayerAction;
    fn ask_yes_no(&mut self, game: &Game, ctx: &AskContext) -> bool;
    fn ask_cards_from_pile(
        &mut self,
        game: &Game,
        ctx: &AskContext,
        pile: PileFlag,
        count: usize,
    ) -> Vec<(PileFlag, usize)>;
    fn ask_shop_card(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize;
    fn ask_enemy_base(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize;
    fn ask_played_ship(&mut self, game: &Game, ctx: &AskContext) -> usize;
}
