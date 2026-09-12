use crate::actions::{AskContext, PlayerAction};
use crate::agent::Agent;
use crate::cards::CardNamed;
use crate::effects::PileFlag;
use crate::game::Game;

pub struct PassBot10000 {}

impl Agent for PassBot10000 {
    fn choose_action(&mut self, game: &Game, actor: usize) -> PlayerAction {
        PlayerAction::EndTurn
    }

    fn ask_yes_no(&mut self, game: &Game, ctx: &AskContext) -> bool {
        false
    }

    fn ask_cards_from_pile(
        &mut self,
        game: &Game,
        ctx: &AskContext,
        pile: PileFlag,
        count: usize,
    ) -> Vec<(PileFlag, usize)> {
        todo!()
    }

    fn ask_shop_card(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize {
        todo!()
    }

    fn ask_enemy_base(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize {
        todo!()
    }

    fn ask_played_ship(
        &mut self,
        game: &Game,
        ctx: &AskContext,
        eligible: &[CardNamed],
    ) -> CardNamed {
        todo!()
    }
}
