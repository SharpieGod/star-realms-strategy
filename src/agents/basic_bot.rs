use crate::{
    abilities::PileFlag,
    actions::{
        AskContext,
        PlayerAction::{BuyCard, PlayAll},
    },
    agent::Agent,
    cards::CARDS,
    game::Game,
};

pub struct BasicBot {}

impl Agent for BasicBot {
    fn choose_action(
        &mut self,
        game: &crate::game::Game,
        actor: usize,
    ) -> crate::actions::PlayerAction {
        dbg!(game.legal_moves(actor)[0]);
        game.legal_moves(actor)[0]
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
        let mut out = Vec::new();
        let player = &game.players[ctx.actor];
        let mut hand_index = 0;
        let mut discard_index = 0;

        for _ in 0..count {
            if pile.contains(PileFlag::HAND) && !player.hand.is_empty() {
                out.push((PileFlag::HAND, hand_index));
                hand_index += 1;
            } else if pile.contains(PileFlag::DISCARD_PILE) && !player.discard_pile.is_empty() {
                out.push((PileFlag::DISCARD_PILE, discard_index));
                discard_index += 1;
            }
        }

        out
    }

    fn ask_shop_card(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize {
        eligible[0]
    }

    fn ask_enemy_base(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize {
        eligible[0]
    }

    fn ask_played_ship(&mut self, game: &Game, ctx: &AskContext, eligible: &[usize]) -> usize {
        eligible[0]
    }
}
