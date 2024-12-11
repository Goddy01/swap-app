use anchor_lang::prelude::*;
// Importing Anchor SPL libraries for handling associated tokens and token operations.
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, // Function to safely close token accounts.
        transfer_checked, // Function to transfer tokens with validation.
        CloseAccount, // Struct to define closing account instructions.
        Mint, // Represents the token mint (currency).
        TokenAccount, // Represents a token account.
        TokenInterface, // Represents the token program interface.
        TransferChecked, // Struct to define transfer instructions.
    },
};

use super::transfer_tokens; // A utility function defined elsewhere for token transfers.
use crate::Offer; // Importing the `Offer` struct, which represents the offer details.

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    // The signer account representing the user taking the offer.
    #[account(mut)]
    pub taker: Signer<'info>,

    // The maker (creator) of the offer. This account is mutable as it may receive tokens.
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    // Token mint for the offered token (A).
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    // Token mint for the wanted token (B).
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    // The taker's token account for the offered token (A).
    // It will be created if it doesn't exist.
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    // The taker's token account for the wanted token (B).
    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // The maker's token account for the wanted token (B).
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // The offer account containing details about the trade.
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
        seeds = [b"offer", maker.key().as_ref(), offer.id.to_le_bytes().as_ref()],
        bump = offer.bump
    )]
    offer: Account<'info, Offer>,

    // The vault holding the tokens offered by the maker.
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // Required Solana programs for system operations.
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// Function to transfer the wanted tokens (B) from the taker to the maker.
pub fn send_wanted_tokens_to_maker(ctx: &Context<TakeOffer>) -> Result<()> {
    transfer_tokens(
        &ctx.accounts.taker_token_account_b, // Source account (taker's token B).
        &ctx.accounts.maker_token_account_b, // Destination account (maker's token B).
        &ctx.accounts.offer.token_b_wanted_amount, // Amount to transfer.
        &ctx.accounts.token_mint_b, // Token mint for B.
        &ctx.accounts.taker, // Signer (taker).
        &ctx.accounts.token_program, // Token program.
    )
}

// Function to withdraw tokens from the vault and close it.
pub fn withdraw_and_close_vault(ctx: Context<TakeOffer>) -> Result<()> {
    // Seeds for generating the vault's PDA.
    let seeds = &[
        b"offer",
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &ctx.accounts.offer.id.to_le_bytes()[..],
        &[ctx.accounts.offer.bump],
    ];
    let signer_seeds = [&seeds[..]];

    // Instruction for transferring tokens from the vault to the taker.
    let accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(), // Source vault.
        to: ctx.accounts.taker_token_account_a.to_account_info(), // Destination account.
        mint: ctx.accounts.token_mint_a.to_account_info(), // Mint for token A.
        authority: ctx.accounts.offer.to_account_info(), // Authority (offer PDA).
    };

    // Creating CPI context for the transfer.
    let cpi_content: CpiContext<TransferChecked> = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    // Performing the transfer.
    transfer_checked(
        cpi_content,
        ctx.accounts.vault.amount, // Amount to transfer.
        ctx.accounts.token_mint_a.decimals, // Token decimal places.
    )?;

    // Instruction for closing the vault.
    let accounts = CloseAccount {
        account: ctx.accounts.vault.to_account_info(), // Vault to close.
        destination: ctx.accounts.taker.to_account_info(), // Recipient of any remaining funds.
        authority: ctx.accounts.offer.to_account_info(), // Authority (offer PDA).
    };

    // Creating CPI context for closing the account.
    let cpi_content = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    // Closing the vault.
    close_account(cpi_content)
}
