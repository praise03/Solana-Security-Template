#![no_std]

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pkey::Pkey,
    ProgramResult,
};
use pinocchio_log::log;

entrypoint!(process_instruction);

p const ID: Pkey = [
    0x0f, 0x1e, 0x6b, 0x14, 0x13, 0xc0, 0x4a, 0x07, 0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee,
    0x19, 0x26, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07, 0x8e, 0xf6, 0xaf, 0x70, 0x37, 0xdc, 0x11, 0xf7,
];

// Simple config struct we'll try to read from account data
#[repr(C)]
p struct NormalConfig {
    p state: u8,      // 1 byte
    p value: u64,     // 8 bytes - requires 8-byte alignment
    p authority: Pkey,  // 32 bytes
}

// Packed version for patterns 2 & 3 - no padding between fields
#[repr(C, packed)]
p struct PackedConfig {
    p state: u8,      // 1 byte
    p value: u64,     // 8 bytes, but now unaligned (starts right after u8)
    p authority: Pkey,  // 32 bytes
}

// Bad struct for pattern 5 - default #[repr(Rust)] means unpredictable layout
#[repr(Rust)]
p struct BadStruct {
    p value: u64,
}

// Main instruction processor - we'll show all 5 patterns here
fn process_instruction(
    _program_id: &Pkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    if accounts.len() < 1 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let account = &accounts[0];  // Account with data to read

    // Beginner note: We'll assume the account data is a byte slice we want to read as structs.
    // But we'll do it UNSAFELY in 5 ways, explaining why it's bad each time.
    // What can go wrong overall: Program panic (crash), wrong values read (incorrect logic, e.g. fake authority allowing fund drain), or undefined behavior () like memory corruption leading to weird bugs or exploits.

    let data = account.try_borrow_data();  // Correct: borrow_data() returns &[u8]

    // Pattern 1: Using transmute() with Unaligned Data
    // Why wrong: transmute() assumes the bytes are perfectly aligned for u64 (must start at memory address divisible by 8). If not,  happens - program might crash or read garbage.
    // What can go wrong: Wrong value read → think it's a high balance when it's not, or fake admin check passes → unauthorized drain of funds.
    if data.len() >= 8 {
        let bytes_slice = &data[0..8];
        let value1: u64 = unsafe { core::mem::transmute(*bytes_slice) };
        log!("Pattern 1 value: {}", value1);
    } else {
        log!("Pattern 1: data too short");
    }

    // Pattern 2: Pointer Casting to Packed Structs
    // Why wrong: Packed structs have no padding, so multi-byte fields like u64 can start at unaligned addresses (e.g. after a u8). Casting creates references to unaligned data → .
    // What can go wrong: Crash or read wrong authority → attacker fakes ownership, drains funds from an account they shouldn't control.
    let packed_ptr = data.as_ptr() as *const PackedConfig;
    let packed = unsafe { &*packed_ptr };
    let value2 = packed.value;  //  if unaligned
    log!("Pattern 2 value: {}", value2);

    // Pattern 3: Direct Field Access on Packed Structs
    // Why wrong: Accessing fields like .value creates a reference to unaligned data → , even if the struct fits in memory.
    // What can go wrong: Temporary unaligned reference causes crash or corruption → program fails to update balances correctly, locking funds or allowing dole-spend.
    let packed = unsafe { &*(data.as_ptr() as *const PackedConfig) };
    let b_ref = &packed.value;  // : reference to unaligned u64
    let b_value = packed.value; // : temporary reference
    log!("Pattern 3 b_value: {}", b_value);

    // Pattern 4: Assuming Alignment Without Verification
    // Why wrong: Casting pointer without checking if it's aligned for the struct (e.g. u64 needs 8-byte alignment) →  if misaligned.
    // What can go wrong: Reads garbage as authority → thinks attacker is admin → allows unauthorized actions like draining funds from a vault.
    let config_ptr = data.as_ptr() as *const NormalConfig;
    let config = unsafe { &*config_ptr };
    let value4 = config.value;
    log!("Pattern 4 value: {}", value4);

    // Pattern 5: Using read_unaligned() Incorrectly
    // Why wrong: read_unaligned() is for unaligned data, but the struct must have predictable layout (#[repr(C)]). With default #[repr(Rust)], layout is undefined → .
    // What can go wrong: Wrong field read → incorrect timestamp or balance calculation → overpay rewards or underpay fees, leading to fund loss for the program.
    let bad_ptr = data.as_ptr() as *const BadStruct;
    let bad = unsafe { bad_ptr.read_unaligned() };
    let value5 = bad.value;
    log!("Pattern 5 value: {}", value5);

    Ok(())
}