use mollusk_svm::{Check, Mollusk};
use solana_sdk::{
    account::Account,
    instruction::Instruction,
    pubkey::Pubkey,
};

use bytemuck::{Pod, Zeroable};

const PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111111111111111");

// Instruction discriminators
const UPDATE_FEE_VULN: u8 = 0;
const UPDATE_FEE_SAFE: u8 = 1;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ConfigState {
    admin: Pubkey,
    protocol_fee: u64,
}

fn serialize_config_state(state: &ConfigState) -> Vec<u8> {
    bytemuck::bytes_of(state).to_vec()
}

fn update_fee_instruction(discriminator: u8, new_fee: u64) -> Vec<u8> {
    let mut data = vec![discriminator];
    data.extend_from_slice(&new_fee.to_le_bytes());
    data
}

#[test]
fn test_update_fee_vulnerable_and_safe() {
    // Initialize Mollusk with the program
    let mut mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/missing_account_checks_pinocchio");

    // Create admin and config PDAs / keys
    let admin = Pubkey::new_unique();
    let config = Pubkey::new_unique();

    // Initial config: admin and a small fee
    let initial_state = ConfigState {
        admin,
        protocol_fee: 10,
    };

    // Add config account with initial state
    mollusk.add_account(
        config,
        Account {
            lamports: 1_000_000,
            data: serialize_config_state(&initial_state),
            owner: PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Add admin account (no data, no signer privilege yet)
    mollusk.add_account(
        admin,
        Account {
            lamports: 0,
            data: vec![],
            owner: Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        },
    );

    // --- VULNERABLE UPDATE (no admin signature) ---
    let vuln_ix = Instruction::new_with_bytes(
        PROGRAM_ID,
        &update_fee_instruction(UPDATE_FEE_VULN, 999),
        vec![
            // config (writable)
            solana_sdk::instruction::AccountMeta::new(config, false),
            // admin (not signer)
            solana_sdk::instruction::AccountMeta::new(admin, false),
        ],
    );

    println!("\n--- Running vulnerable update (no signer) ---");

    mollusk.process_and_validate_instruction(
        &vuln_ix,
        &[],
        &[Check::success()],
    );

    // Check updated state
    let updated_config = mollusk.get_account(&config).unwrap();
    let updated_state: &ConfigState = bytemuck::from_bytes(&updated_config.data);

    println!("VULNERABLE: protocol_fee now {}", updated_state.protocol_fee);
    assert_eq!(updated_state.protocol_fee, 999);

    // --- SAFE UPDATE (still no admin signature) ---
    let safe_ix_no_sig = Instruction::new_with_bytes(
        PROGRAM_ID,
        &update_fee_instruction(UPDATE_FEE_SAFE, 555),
        vec![
            solana_sdk::instruction::AccountMeta::new(config, false),
            solana_sdk::instruction::AccountMeta::new(admin, false),
        ],
    );

    println!("\n--- Running safe update (no signer) ---");

    mollusk.process_and_validate_instruction(
        &safe_ix_no_sig,
        &[],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // Re-fetch config to make sure it didn’t change
    let unchanged_config = mollusk.get_account(&config).unwrap();
    let unchanged_state: &ConfigState = bytemuck::from_bytes(&unchanged_config.data);

    println!("SAFE without signer: protocol_fee remains {}", unchanged_state.protocol_fee);
    assert_eq!(unchanged_state.protocol_fee, 999);

    // --- SAFE UPDATE (with admin signature) ---
    println!("\n--- Running safe update (with signer) ---");

    mollusk.process_and_validate_instruction(
        &safe_ix_no_sig,
        &[(admin, mollusk.get_account(&admin).unwrap().clone())],
        &[Check::success()],
    );

    let final_config = mollusk.get_account(&config).unwrap();
    let final_state: &ConfigState = bytemuck::from_bytes(&final_config.data);

    println!("SAFE with signer: protocol_fee now {}", final_state.protocol_fee);
    assert_eq!(final_state.protocol_fee, 555);
}