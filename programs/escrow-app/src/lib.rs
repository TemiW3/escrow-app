#![allow(unexpected_cfgs)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GP2mZSjyRK3tX151UHENPLyHbTRqfz7dvYPrQpqJrHzv");

#[program]
pub mod escrow_app {
    use super::*;

    pub fn make_offer(ctx: Context<MakeOffer>, id: u64, token_a_offered_amount: u64, token_b_wanted_amount: u64) -> Result<()> {
        instructions::make_offer::send_offered_tokens_to_escrow(&ctx, token_a_offered_amount)?;
        instructions::make_offer::save_offer(ctx, id, token_b_wanted_amount)
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        instructions::accept_offer::transfer_tokens_to_maker(&ctx)?;
        instructions::accept_offer::withdraw_and_close_escrow(ctx)
    }
}


