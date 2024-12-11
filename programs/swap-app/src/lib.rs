pub mod constants; // Module for storing reusable constants.
pub mod error; // Module for custom error definitions.
pub mod instructions; // Module containing the logic for program instructions.
pub mod state; // Module defining program-specific data structures and states.

use anchor_lang::prelude::*; // Importing Anchor framework essentials.

// Re-exporting modules for easy access in the program.
pub use constants::*;
pub use instructions::*;
pub use state::*;

// Declare the unique program ID. This must match the deployed program's ID.
declare_id!("35B6fNAgPfqeW9d9qANsQrECSYFDxoobTo3CTrZJxAvJ");

#[program] // Marks the start of the program module.
pub mod swap_app {
    use super::*; // Import everything from the parent scope.

    /// Creates a new offer by sending tokens to a vault and saving offer details.
    /// 
    /// # Arguments
    /// - `ctx`: Context containing accounts required to execute the instruction.
    /// - `id`: Unique identifier for the offer.
    /// - `token_a_offered_amount`: Amount of Token A being offered.
    /// - `token_b_offered_amount`: Amount of Token B being requested in return.
    pub fn make_offer(
        ctx: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_offered_amount: u64,
    ) -> Result<()> {
        // Step 1: Transfer offered tokens (Token A) from the maker's account to the program vault.
        instructions::make_offer::send_offered_tokens_to_vault(&ctx, token_a_offered_amount)?;

        // Step 2: Save the details of the offer (id, requested amount, etc.) in the program state.
        instructions::make_offer::save_offer(ctx, id, token_b_offered_amount)
    }

    /// Accepts an existing offer by transferring tokens and closing the vault.
    ///
    /// # Arguments
    /// - `ctx`: Context containing accounts required to execute the instruction.
    pub fn take_offer(ctx: Context<TakeOffer>) -> Result<()> {
        // Step 1: Transfer the requested amount of Token B from the taker's account to the maker's account.
        instructions::take_offer::send_wanted_tokens_to_maker(&ctx)?;

        // Step 2: Withdraw the offered tokens (Token A) from the vault to the taker's account
        // and close the vault account.
        instructions::take_offer::withdraw_and_close_vault(ctx)
    }
}
