// Import cryptographic utilities to generate random bytes.
import { randomBytes } from "node:crypto";

// Importing Anchor framework to interact with Solana programs.
import * as anchor from "@coral-xyz/anchor";

// Importing Big Number support (BN) and program types from Anchor.
import { BN, type Program } from "@coral-xyz/anchor";

// Importing token utilities and constants from the Solana SPL Token library.
import {
  TOKEN_2022_PROGRAM_ID, // Identifier for the 2022 Token Program.
  type TOKEN_PROGRAM_ID, // Default Token Program Identifier type.
  getAssociatedTokenAddressSync, // Function to derive associated token addresses.
} from "@solana/spl-token";

// Import Solana web3.js utilities, like lamports and public keys.
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";

// Assertion library for test validations.
import { assert } from "chai";

// Importing custom helper utilities for testing and keypair generation.
import {
  confirmTransaction, // Function to confirm transactions on-chain.
  createAccountsMintsAndTokenAccounts, // Helper to create accounts, mints, and token accounts.
  makeKeypairs, // Utility to generate random keypairs.
} from "@solana-developers/helpers";

import type { SwapApp } from "../target/types/swap_app";

// Use either the legacy Token Program or the 2022 Token Extensions Program.
const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
  TOKEN_2022_PROGRAM_ID;

// Define a time constant in milliseconds for conversions and thresholds.
const SECONDS = 1000;

// Set the threshold for marking tests as "slow." Tests beyond this time are flagged.
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;

// Function to generate a random big number using cryptographic randomness.
const getRandomBigNumber = (size = 8) => {
  return new BN(randomBytes(size));
};

// Describe the suite of tests for the "swap" program.
describe("swap", async () => {
  // Set up the Anchor environment using configuration from Anchor.toml.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Get the wallet's keypair (payer) to fund transactions.
  const user = (provider.wallet as anchor.Wallet).payer;
  const payer = user;

  // Connection to the Solana cluster for sending transactions.
  const connection = provider.connection;

  // Reference to the compiled SwapApp program.
  const program = anchor.workspace.Swap as Program<SwapApp>;

  // Object to store account references used across tests.
  const accounts: Record<string, PublicKey> = {
    tokenProgram: TOKEN_PROGRAM, // Set the token program (default or 2022).
  };

  // Initialize variables for user keypairs and token mints.
  let alice: anchor.web3.Keypair;
  let bob: anchor.web3.Keypair;
  let tokenMintA: anchor.web3.Keypair;
  let tokenMintB: anchor.web3.Keypair;

  // Generate keypairs for Alice, Bob, and the token mints.
  [alice, bob, tokenMintA, tokenMintB] = makeKeypairs(4);

  // Define the offered and requested token amounts.
  const tokenAOfferedAmount = new BN(1_000_000);
  const tokenBWantedAmount = new BN(1_000_000);

  // Before all tests, create accounts, mints, and token accounts.
  before(
    "Creates Alice and Bob accounts, 2 token mints, and associated token accounts for both users",
    async () => {
      const usersMintsAndTokenAccounts =
        await createAccountsMintsAndTokenAccounts(
          [
            // Define initial token balances for Alice and Bob.
            [
              1_000_000_000, // Alice starts with Token A.
              0, // Alice has no Token B initially.
            ],
            [
              0, // Bob has no Token A initially.
              1_000_000_000, // Bob starts with Token B.
            ],
          ],
          1 * LAMPORTS_PER_SOL, // Fund each user with 1 SOL for fees.
          connection,
          payer
        );

      // Extract users, mints, and token accounts for use in tests.
      const users = usersMintsAndTokenAccounts.users;
      alice = users[0];
      bob = users[1];

      const mints = usersMintsAndTokenAccounts.mints;
      tokenMintA = mints[0];
      tokenMintB = mints[1];

      const tokenAccounts = usersMintsAndTokenAccounts.tokenAccounts;

      // Save token accounts for future reference.
      const aliceTokenAccountA = tokenAccounts[0][0];
      const aliceTokenAccountB = tokenAccounts[0][1];
      const bobTokenAccountA = tokenAccounts[1][0];
      const bobTokenAccountB = tokenAccounts[1][1];

      accounts.maker = alice.publicKey;
      accounts.taker = bob.publicKey;
      accounts.tokenMintA = tokenMintA.publicKey;
      accounts.makerTokenAccountA = aliceTokenAccountA;
      accounts.takerTokenAccountA = bobTokenAccountA;
      accounts.tokenMintB = tokenMintB.publicKey;
      accounts.makerTokenAccountB = aliceTokenAccountB;
      accounts.takerTokenAccountB = bobTokenAccountB;
    }
  );

  // Test for Alice making an offer.
  it("Puts the tokens Alice offers into the vault when Alice makes an offer", async () => {
    const offerId = getRandomBigNumber(); // Generate a random offer ID.

    // Derive the offer and vault account addresses.
    const offer = PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"), // Prefix for seeds.
        accounts.maker.toBuffer(), // Maker's public key.
        offerId.toArrayLike(Buffer, "le", 8), // Offer ID as little-endian.
      ],
      program.programId
    )[0];

    const vault = getAssociatedTokenAddressSync(
      accounts.tokenMintA, // Token A mint.
      offer, // Offer account as the owner.
      true, // Create if not found.
      TOKEN_PROGRAM
    );

    // Save derived accounts for reuse.
    accounts.offer = offer;
    accounts.vault = vault;

    // Call the `makeOffer` method on the program.
    const transactionSignature = await program.methods
      .makeOffer(offerId, tokenAOfferedAmount, tokenBWantedAmount)
      .accounts({ ...accounts })
      .signers([alice]) // Alice signs the transaction.
      .rpc();

    await confirmTransaction(connection, transactionSignature); // Confirm the transaction.

    // Validate the vault contains the correct token amount.
    const vaultBalanceResponse = await connection.getTokenAccountBalance(vault);
    const vaultBalance = new BN(vaultBalanceResponse.value.amount);
    assert(vaultBalance.eq(tokenAOfferedAmount));

    // Validate the Offer account contains the correct data.
    const offerAccount = await program.account.offer.fetch(offer);
    assert(offerAccount.maker.equals(alice.publicKey));
    assert(offerAccount.tokenMintA.equals(accounts.tokenMintA));
    assert(offerAccount.tokenMintB.equals(accounts.tokenMintB));
    assert(offerAccount.tokenBWantedAmount.eq(tokenBWantedAmount));
  }).slow(ANCHOR_SLOW_TEST_THRESHOLD); // Mark this test as slow if it exceeds the threshold.

  // Test for Bob taking an offer.
  it("Puts the tokens from the vault into Bob's account, and gives Alice Bob's tokens, when Bob takes an offer", async () => {
    const transactionSignature = await program.methods
      .takeOffer()
      .accounts({ ...accounts })
      .signers([bob]) // Bob signs the transaction.
      .rpc();

    await confirmTransaction(connection, transactionSignature); // Confirm the transaction.

    // Validate that Bob's account received the offered tokens.
    const bobTokenAccountBalanceAfterResponse =
      await connection.getTokenAccountBalance(accounts.takerTokenAccountA);
    const bobTokenAccountBalanceAfter = new BN(
      bobTokenAccountBalanceAfterResponse.value.amount
    );
    assert(bobTokenAccountBalanceAfter.eq(tokenAOfferedAmount));

    // Validate that Alice's account received the wanted tokens.
    const aliceTokenAccountBalanceAfterResponse =
      await connection.getTokenAccountBalance(accounts.makerTokenAccountB);
    const aliceTokenAccountBalanceAfter = new BN(
      aliceTokenAccountBalanceAfterResponse.value.amount
    );
    assert(aliceTokenAccountBalanceAfter.eq(tokenBWantedAmount));
  }).slow(ANCHOR_SLOW_TEST_THRESHOLD); // Mark this test as slow if it exceeds the threshold.
});
