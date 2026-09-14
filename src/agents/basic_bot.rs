use crate::{
    actions::PlayerAction::{BuyCard, PlayAll},
    agent::Agent,
    cards::CARDS,
};

pub struct BasicBot {}

impl Agent for BasicBot {
    fn choose_action(
        &mut self,
        game: &crate::game::Game,
        actor: usize,
    ) -> crate::actions::PlayerAction {
        let player = &game.players[actor];

        if !player.hand.is_empty() {
            return PlayAll;
        }

        // buy the first card you can

        if let Some(shop_index) = game.shop.iter().enumerate().find_map(|(i, c)| {
            c.is_some_and(|c| CARDS[&c].cost <= player.trade)
                .then_some(i)
        }) {
            return BuyCard(shop_index);
        }

        PlayAll
    }

    fn ask_yes_no(&mut self, game: &crate::game::Game, ctx: &crate::actions::AskContext) -> bool {
        false
    }

    fn ask_cards_from_pile(
        &mut self,
        game: &crate::game::Game,
        ctx: &crate::actions::AskContext,
        pile: crate::abilities::PileFlag,
        count: usize,
    ) -> Vec<(crate::abilities::PileFlag, usize)> {
        todo!()
    }

    fn ask_shop_card(
        &mut self,
        game: &crate::game::Game,
        ctx: &crate::actions::AskContext,
        eligible: &[usize],
    ) -> usize {
        todo!()
    }

    fn ask_enemy_base(
        &mut self,
        game: &crate::game::Game,
        ctx: &crate::actions::AskContext,
        eligible: &[usize],
    ) -> usize {
        todo!()
    }

    fn ask_played_ship(
        &mut self,
        game: &crate::game::Game,
        ctx: &crate::actions::AskContext,
        eligible: &[usize],
    ) -> usize {
        todo!()
    }
}
