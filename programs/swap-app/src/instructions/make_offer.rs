use anchor_lang::prelude::*;
// Importing Anchor SPL libraries for handling associated tokens and token operations.
use anchor_spl::{
    associated_token::AssociatedToken, // Provides functions for working with associated token accounts.
    token_interface::{
        Mint,           // Represents a token mint (defines the type of a token).
        TokenAccount,   // Represents a token account that holds tokens.
        TokenInterface, // Interface for interacting with the token program.
        TransferChecked, // Struct for performing a safe token transfer.
        transfer_checked // Function to execute a safe token transfer.
    }
};
use crate::{Offer, ANCHOR_DISCRIMINATION};

use super::transfer_tokens; // Imports custom definitions for the `Offer` struct and the `ANCHOR_DISCRIMINATION` constant.

/// Defines the context for the `MakeOffer` instruction.
/// This structure includes all accounts and programs involved in the transaction.
#[derive(Accounts)]
#[instruction(id: u64)] // Specifies that this instruction requires an `id` parameter.
pub struct MakeOffer<'info> {
    /// The maker of the offer, who must sign the transaction.
    #[account(mut)]
    pub maker: Signer<'info>, // A mutable signer account for the maker.

    /// The mint account for token A (type of token to be offered).
    #[account(mint::token_program=token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>, // Token A mint interface.

    /// The mint account for token B (type of token wanted in exchange).
    #[account(mint::token_program=token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>, // Token B mint interface.

    /// The maker's associated token account for token A.
    /// This account will hold the tokens to be offered.
    #[account(
        mut,
        associated_token::mint=token_mint_a, // Ensures this account matches token A's mint.
        associated_token::authority=maker,  // Ensures the maker is the authority over this account.
        associated_token::token_program=token_program // Specifies the token program used.
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>, // Maker's token account for token A.

    /// The offer account to be created, which represents the details of the offer.
    /// This account is initialized with specific seeds and a space requirement.
    #[account(
        init, // Indicates the account is being initialized.
        payer=maker, // The maker pays for the initialization.
        space = ANCHOR_DISCRIMINATION + Offer::INIT_SPACE, // Total space for the account (Anchor discriminator + custom fields).
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()], // PDA seeds (program-derived address).
        bump // Auto-generates a bump seed for the PDA.
    )]
    pub offer: Account<'info, Offer>, // The on-chain offer account.

    /// A vault account to hold the offered tokens.
    /// This is an associated token account owned by the offer account.
    #[account(
        init, // Indicates the vault account is being initialized.
        payer=maker, // The maker pays for the initialization.
        associated_token::mint=token_mint_a, // The vault corresponds to token A.
        associated_token::authority=offer, // The offer account is the authority over this vault.
        associated_token::token_program=token_program // Specifies the token program used.
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, // The vault account for holding offered tokens.

    /// Solana's system program for account creation and lamport transfers.
    pub system_program: Program<'info, System>, // System program required for initializing accounts.

    /// The token program for handling token operations.
    pub token_program: Interface<'info, TokenInterface>, // Token interface for program interactions.

    /// The associated token program for creating associated token accounts.
    pub associated_token_program: Program<'info, AssociatedToken>, // Program for associated token accounts.
}

/// This function transfers the offered tokens from the maker's account to the vault.
/// It uses the `transfer_tokens` utility to perform the token transfer operation securely.
pub fn send_offered_tokens_to_vault(
    ctx: &Context<MakeOffer>, // The context containing all accounts and data for the `MakeOffer` instruction.
    token_a_offered_amount: u64, // The amount of token A being offered by the maker.
) -> Result<()> {
    // Call the `transfer_tokens` function to execute the token transfer.
    transfer_tokens(
        &ctx.accounts.maker_token_account_a, // Source account: Maker's token account holding the offered tokens.
        &ctx.accounts.vault, // Destination account: The vault account for securely holding the offered tokens.
        &token_a_offered_amount, // Amount of tokens to transfer from the maker to the vault.
        &ctx.accounts.token_mint_a, // Mint account for the token being transferred (token A).
        &ctx.accounts.maker, // The authority signing the transaction (the maker of the offer).
        &ctx.accounts.token_program // The token program interface handling the transfer operation.
    )
}
