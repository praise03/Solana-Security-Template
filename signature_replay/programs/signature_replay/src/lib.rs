use anchor_lang::prelude::*;
use solana_program::secp256k1_recover;

declare_id!("H1xQDjtQUEPTrS3WtzSvXYXUPGZUttr6jx9PPF7f7XbZ");
/*
    =========================================================================
    SIGNATURE REPLAY VULNERABILITY (SECP256K1)
    =========================================================================

    This program demonstrates a common replay vulnerability in
    secp256k1-based authorization flows.

*/

#[program]
pub mod signature_replay_example {
    use super::*;

    /*
        ---------------------------------------------------------------------
        VULNERABLE INSTRUCTION
        ---------------------------------------------------------------------

        A valid signature authorizes an action.
        The signature can be reused indefinitely.
    */
    pub fn authorize_vulnerable(
        _ctx: Context<VulnerableAuth>,
        message: [u8; 32],
        signature: [u8; 64],
        recovery_id: u8,
        expected_pubkey: [u8; 64],
    ) -> Result<()> {
        let recovered_pubkey = secp256k1_recover(
            &message,
            recovery_id,
            &signature,
        )
        .map_err(|_| error!(ErrorCode::InvalidSignature))?;

        require!(
            recovered_pubkey.to_bytes() == expected_pubkey,
            ErrorCode::InvalidSignature
        );

        /*
            BUG:

            Nothing ties this signature to a single use.
            The same signature can be replayed forever.
        */

        Ok(())
    }

    /*
        ---------------------------------------------------------------------
        SAFE INSTRUCTION
        ---------------------------------------------------------------------

        A nonce is included in the signed message and tracked on-chain.
        Each signature can only be used once.
    */
    pub fn authorize_safe(
        ctx: Context<SafeAuth>,
        message: [u8; 32],
        signature: [u8; 64],
        recovery_id: u8,
        expected_pubkey: [u8; 64],
        nonce: u64,
    ) -> Result<()> {
        let recovered_pubkey = secp256k1_recover(
            &message,
            recovery_id,
            &signature,
        )
        .map_err(|_| error!(ErrorCode::InvalidSignature))?;

        require!(
            recovered_pubkey.to_bytes() == expected_pubkey,
            ErrorCode::InvalidSignature
        );

        /*
            CRITICAL DEFENSE:

            Enforce single-use by rejecting reused nonces.
        */
        let nonce_state = &mut ctx.accounts.nonce_state;

        require!(
            nonce > nonce_state.last_nonce,
            ErrorCode::NonceAlreadyUsed
        );

        //update nonce in storage
        nonce_state.last_nonce = nonce;

        Ok(())
    }
}

/*
    -------------------------------------------------------------------------
    ACCOUNTS
    -------------------------------------------------------------------------
*/

#[derive(Accounts)]
pub struct VulnerableAuth {}

#[derive(Accounts)]
pub struct SafeAuth {
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + NonceState::SIZE
    )]
    pub nonce_state: Account<'info, NonceState>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/*
    -------------------------------------------------------------------------
    STATE
    -------------------------------------------------------------------------
*/

#[account]
pub struct NonceState {
    /*
        Stores the last nonce accepted.
        Prevents signature replay.
    */
    pub last_nonce: u64,
}

impl NonceState {
    pub const SIZE: usize = 8;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid secp256k1 signature")]
    InvalidSignature,

    #[msg("Signature nonce already used")]
    NonceAlreadyUsed,
}
