#![cfg(test)]

use mollusk::*;
use solana_sdk::{pubkey::Pubkey, account::Account, signature::Keypair};

#[test]
fn test_update_fee_vulnerability() {
    // -----------------------------
    // 1. Initialize the VM
    // -----------------------------
    let mut vm = Vm::new();

    // Program ID of the compiled Pinocchio program
    let program_id = Pubkey::new_unique();

    // Include compiled .so for the program
    vm.add_program(
        program_id,
        include_bytes!("../target/deploy/unsafe_misuse_pinocchio.so"),
    );

    // -----------------------------
    // 2. Create admin and config accounts
    // -----------------------------
    let admin = Keypair::new();
    let mut config_account = Account::new(
        1_000_000,                     // lamports
        core::mem::size_of::<[u8; 32 + 8]>() as u64, // space for ConfigState (Pubkey + u64)
        &program_id,
    );

    // Initialize the config state manually (for test purposes)
    let mut config_data = config_account.data.as_mut_slice();
    // Set admin pubkey
    config_data[..32].copy_from_slice(admin.pubkey().as_ref());
    // protocol_fee = 0
    config_data[32..40].copy_from_slice(&0u64.to_le_bytes());

    let mut accounts = vec![
        (Pubkey::new_unique(), config_account), // config account
        (admin.pubkey(), Account::new(1_000_000, 0, &program_id)), // admin account
    ];

    // -----------------------------
    // 3. Prepare instruction data
    // -----------------------------
    let new_fee: u64 = 500; // arbitrary new fee

    let instruction_vulnerable = vec![0u8]; // discriminator 0 = vulnerable
    let mut instruction_vulnerable_data = instruction_vulnerable.clone();
    instruction_vulnerable_data.extend_from_slice(&new_fee.to_le_bytes());

    let instruction_safe = vec![1u8]; // discriminator 1 = safe
    let mut instruction_safe_data = instruction_safe.clone();
    instruction_safe_data.extend_from_slice(&new_fee.to_le_bytes());

    // -----------------------------
    // 4. Test vulnerable update
    // -----------------------------
    // Note: we do NOT mark the admin as a signer to show the vulnerability
    println!("Testing VULNERABLE update (admin not signer)...");
    let res = vm.execute(program_id, accounts.clone(), instruction_vulnerable_data);

    // Vulnerable allows update even without admin signature
    assert!(res.is_ok());
    println!("VULNERABLE update succeeded (should be prevented in SAFE code)");

    // -----------------------------
    // 5. Test safe update
    // -----------------------------
    // Mollusk requires marking the account as signer for the safe instruction
    println!("Testing SAFE update (requires admin signer)...");
    let mut accounts_safe = accounts.clone();

    // Mark the admin as signer in Mollusk
    accounts_safe[1].1.is_signer = true;

    let res_safe = vm.execute(program_id, accounts_safe, instruction_safe_data);

    assert!(res_safe.is_ok());
    println!("SAFE update succeeded (admin signer required)");

    // -----------------------------
    // 6. Test SAFE fails if admin not signer
    // -----------------------------
    println!("Testing SAFE update fails without signer...");
    let res_fail = vm.execute(program_id, accounts.clone(), instruction_safe_data);
    assert!(res_fail.is_err());
    println!("SAFE update correctly rejected when admin not signer");
}