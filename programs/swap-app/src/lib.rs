pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("35B6fNAgPfqeW9d9qANsQrECSYFDxoobTo3CTrZJxAvJ");

#[program]
pub mod swap_app {
    use super::*;

    pub fn make_offer(ctx: Context<MakeOffer>, id:u64, token_a_offered_amount: u64, token_b_offered_amount: u64) -> Result<()> {
        instructions::make_offer::send_offered_tokens_to_vault(&ctx, token_a_offered_amount)?;
        instructions::make_offer::save_offer(ctx, id, token_b_offered_amount);
    }
}
