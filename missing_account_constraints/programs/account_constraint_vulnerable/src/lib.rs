use anchor_lang::prelude::*;

declare_id!("9WHbb8dVjqxDrfAQQSVqiKSe1tdKmki8qvhAmd6HcnL2");

#[program]
pub mod account_constraint_vulnerable {
    use super::*;

    /// Initialize a vault PDA owned by `owner` with an initial logical balance.
    pub fn initialize(ctx: Context<Initialize>, initial_balance: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.owner.key();
        vault.balance = initial_balance;
        Ok(())
    }

    /// Vulnerable withdraw: DOES NOT CHECK that caller == vault.owner.
    /// Any signer can call this and reduce the vault.balance.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // basic check against stored balance
        require!(vault.balance >= amount, ErrorCode::InsufficientFunds);

        // BUG: no owner check — any signer passed as `caller` can trigger this
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

/// VULNERABLE: withdraw context does NOT require the signer to be the owner
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,

    /// Any signer can be passed in as `caller`
    pub caller: Signer<'info>,
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
