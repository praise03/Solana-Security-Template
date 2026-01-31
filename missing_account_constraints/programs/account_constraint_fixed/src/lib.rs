use anchor_lang::prelude::*;

declare_id!("8inSy19nzg5sNawf35TryfVRxzjBtJYwaz8WuNfJ99Nb");

#[program]
pub mod account_constraint_fixed {
    use super::*;

    /// Initialize a vault PDA owned by `owner` with an initial logical balance.
    pub fn initialize(ctx: Context<Initialize>, initial_balance: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.owner.key();
        vault.balance = initial_balance;
        Ok(())
    }

    /// FIXED: withdraw enforces that the signer is the `owner` recorded in the vault.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // ensure only owner can call (Anchor enforces has_one at deserialization)
        require!(vault.balance >= amount, ErrorCode::InsufficientFunds);

        vault.balance = vault.balance.checked_sub(amount).unwrap();
        Ok(())
    }
}

/* ---------- Accounts / State ---------- */

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// PDA: seeds = ["vault", owner_pubkey]
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 8, // discriminator + owner Pubkey + u64
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    /// The wallet that becomes the owner and pays rent for the PDA.
    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// FIXED: `has_one = owner` enforces vault.owner == owner.key()
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, has_one = owner)]
    pub vault: Account<'info, Vault>,

    /// The signer claiming to be the owner
    pub owner: Signer<'info>,
}

/* Vault structure */
#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
}

/* Errors */
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds")]
    InsufficientFunds,
}

