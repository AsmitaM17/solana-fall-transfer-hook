#[allow(dead_code)]
mod helpers;

use anchor_lang::solana_program::system_instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use helpers::{
    setup,
    setup_mint_and_extra_metas,
    create_ata,
    mint_tokens,
    build_transfer_with_hook_ix,
    send_ix,
};

#[test]
fn test_two_wallets_can_transfer_1000000_in_same_hour() {
    let (mut svm, payer, program_id) = setup();

    // Create the mint and initialize the rate limit for the first wallet.
    let mint = Keypair::new();
    setup_mint_and_extra_metas(&mut svm, &payer, &mint, &program_id);

    // Create a second wallet and give it SOL.
    let second_wallet = Keypair::new();

    let airdrop_ix = system_instruction::transfer(
        &payer.pubkey(),
        &second_wallet.pubkey(),
        1_000_000_000,
    );

    send_ix(
        &mut svm,
        airdrop_ix,
        &payer,
        &[&payer],
    );

    // Create token accounts for both wallets.
    let first_ata = create_ata(
        &mut svm,
        &payer,
        &payer.pubkey(),
        &mint.pubkey(),
    );

    let second_ata = create_ata(
        &mut svm,
        &payer,
        &second_wallet.pubkey(),
        &mint.pubkey(),
    );

    // Create a destination ATA for each transfer.
    let first_destination_wallet = Keypair::new();
    let second_destination_wallet = Keypair::new();

    let first_destination_ata = create_ata(
        &mut svm,
        &payer,
        &first_destination_wallet.pubkey(),
        &mint.pubkey(),
    );

    let second_destination_ata = create_ata(
        &mut svm,
        &payer,
        &second_destination_wallet.pubkey(),
        &mint.pubkey(),
    );

    // Mint 1,000,000 tokens to each wallet.
    mint_tokens(
        &mut svm,
        &payer,
        &mint.pubkey(),
        &first_ata,
        1_000_000,
    );

    mint_tokens(
        &mut svm,
        &payer,
        &mint.pubkey(),
        &second_ata,
        1_000_000,
    );

    // First wallet transfers 1,000,000.
    let first_transfer = build_transfer_with_hook_ix(
        &first_ata,
        &first_destination_ata,
        &mint.pubkey(),
        &payer.pubkey(),
        &program_id,
        1_000_000,
        0,
    );

    send_ix(
        &mut svm,
        first_transfer,
        &payer,
        &[&payer],
    );

    // Initialize the rate limit for the SECOND wallet.
    //
    // This is the important part of the challenge:
    // wallet #2 must have its own rate-limit PDA.
    helpers::initialize_rate_limit(
        &mut svm,
        &second_wallet,
        &mint,
        &program_id,
    );

    // Second wallet transfers 1,000,000 during the SAME hour.
    let second_transfer = build_transfer_with_hook_ix(
        &second_ata,
        &second_destination_ata,
        &mint.pubkey(),
        &second_wallet.pubkey(),
        &program_id,
        1_000_000,
        0,
    );

    send_ix(
        &mut svm,
        second_transfer,
        &second_wallet,
        &[&second_wallet],
    );
}