// Importing necessary libraries and modules from Anchor for Solana smart contract development.
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, // For handling associated token accounts.
    token_interface::{Mint, TokenAccount, TokenInterface}, // Interfaces for interacting with tokens.
};

// Importing custom modules and constants.
use crate::{Offer, ANCHOR_DISCRIMINATOR}; // `Offer` is a custom struct, and `ANCHOR_DISCRIMINATOR` ensures unique account identification.

use super::transfer_tokens; // Function to handle token transfers between accounts.

/// Context structure for the `MakeOffer` instruction. This defines the accounts involved.
#[derive(Accounts)]
#[instruction(id: u64)] // `id` is an additional parameter passed to this instruction.
pub struct MakeOffer<'info> {
    #[account(mut)] // Signer account, mutable because the transaction might alter its state.
    pub maker: Signer<'info>,

    // Token Mint A account (immutable) associated with the token program.
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    // Token Mint B account (immutable) associated with the token program.
    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    // Maker's token account for Token A, associated with the `maker` authority.
    #[account(
        mut, // Mutable because tokens will be deducted from this account.
        associated_token::mint = token_mint_a, // The mint for this token account is Token A.
        associated_token::authority = maker, // Authority over this account is the maker.
        associated_token::token_program = token_program // Program that governs this account.
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    // Offer account, initialized during the transaction.
    #[account(
        init, // Creates a new account.
        payer = maker, // Maker pays for the initialization cost.
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE, // Allocating space for the Offer struct.
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()], // PDA seeds for uniqueness.
        bump // Automatically calculates the bump for the PDA.
    )]
    pub offer: Account<'info, Offer>,

    // Vault account to hold the tokens being offered, associated with the Offer account.
    #[account(
        init, // Creates a new account.
        payer = maker, // Maker pays for the initialization cost.
        associated_token::mint = token_mint_a, // The mint for this token account is Token A.
        associated_token::authority = offer, // Authority over this account is the offer account.
        associated_token::token_program = token_program // Program that governs this account.
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // Required system program for account creation.
    pub system_program: Program<'info, System>,
    
    // Token program interface for handling token operations.
    pub token_program: Interface<'info, TokenInterface>,
    
    // Associated token program for creating associated token accounts.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// Transfers the offered tokens from the maker's account to the vault.
/// `token_a_offered_amount` specifies the amount of tokens to transfer.
pub fn send_offered_tokens_to_vault(
    context: &Context<MakeOffer>, // Context containing all the accounts involved.
    token_a_offered_amount: u64, // Amount of Token A to transfer.
) -> Result<()> {
    transfer_tokens(
        &context.accounts.maker_token_account_a, // Source account: Maker's token account.
        &context.accounts.vault, // Destination account: Vault.
        &token_a_offered_amount, // Amount to transfer.
        &context.accounts.token_mint_a, // Mint associated with Token A.
        &context.accounts.maker, // Authority over the source account.
        &context.accounts.token_program, // Token program handling the transfer.
    )
}

/// Saves the offer details into the `Offer` account.
/// `id` is the unique identifier for the offer.
/// `token_b_wanted_amount` specifies the amount of Token B the maker wants in exchange.
pub fn save_offer(
    context: Context<MakeOffer>, // Context containing all the accounts involved.
    id: u64, // Unique identifier for the offer.
    token_b_wanted_amount: u64, // Desired amount of Token B.
) -> Result<()> {
    // Populate the `Offer` account with the provided details.
    context.accounts.offer.set_inner(Offer {
        id, // Offer ID.
        maker: context.accounts.maker.key(), // Maker's public key.
        token_mint_a: context.accounts.token_mint_a.key(), // Public key of Token A mint.
        token_mint_b: context.accounts.token_mint_b.key(), // Public key of Token B mint.
        token_b_wanted_amount, // Amount of Token B wanted.
        bump: context.bumps.offer, // Bump for the Offer PDA.
    });
    Ok(()) // Indicate success.
}
