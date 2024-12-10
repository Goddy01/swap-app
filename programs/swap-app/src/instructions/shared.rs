// Import necessary modules from the Anchor framework
// anchor_lang::prelude::* imports core Anchor framework types and traits
use anchor_lang::prelude::*;

// Import SPL Token Interface related types and functions
// This allows interaction with Solana Program Library (SPL) token programs
use anchor_spl::
    token_interface::{
        Mint,           // Represents a token mint (token type)
        TokenAccount,   // Represents a token account
        TokenInterface, // Interface for token program interactions
        TransferChecked, // Struct for checked token transfers
        transfer_checked // Function to perform a checked token transfer
    };

// Function to transfer tokens with additional safety checks
// Generic lifetime 'info ensures all referenced accounts live for the same duration
pub fn transfer_tokens<'info>(
    // Source token account for the transfer
    from: &InterfaceAccount<'info, TokenAccount>,
    
    // Destination token account for the transfer
    to: &InterfaceAccount<'info, TokenAccount>,
    
    // Amount of tokens to transfer
    amount: &u64,
    
    // Mint (token type) information for decimals and validation
    mint: &InterfaceAccount<'info, Mint>,
    
    // Account authorized to perform the transfer
    authority: &Signer<'info>,
    
    // Token program interface for performing the transfer
    token_program: &Interface<'info, TokenInterface>
) -> Result<()> {
    // Create a TransferChecked struct with required account information
    // This prepares the context for a cross-program invocation (CPI)
    let transfer_account_options = TransferChecked {
        from: from.to_account_info(),     // Source token account
        to: to.to_account_info(),         // Destination token account
        mint: mint.to_account_info(),     // Mint information for validation
        authority: authority.to_account_info() // Account authorizing the transfer
    };

    // Create a Cross-Program Invocation (CPI) context
    // This allows the current program to call the token program
    let cpi_context = CpiContext::new(
        token_program.to_account_info(), // Token program to invoke
        transfer_account_options         // Transfer parameters
    );

    // Perform a checked token transfer
    // Checked transfer ensures:
    // 1. Correct mint is used
    // 2. Sufficient balance in source account
    // 3. Respects token decimal places
    transfer_checked(
        cpi_context,    // CPI context with transfer details
        *amount,        // Amount to transfer (dereferenced)
        mint.decimals   // Number of decimal places for the token
    )
}