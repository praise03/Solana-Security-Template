use anchor_lang::prelude::*;

declare_id!("484vpDiuhXAP7mAgVmUWuxspY6dTA5d9QDuv17JXHDEa");

#[program]
pub mod arithmetic_fixed {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.balance = 0;
        vault.owner = *ctx.accounts.user.key;
        msg!("Vault initialized safely with balance 0");
        Ok(())
    }

    // Fixed: Use checked_add to prevent overflow.
    // Educational: If addition would exceed u64::MAX, reverts with Overflow error.
    pub fn deposit(ctx: Context<DepositWithdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.balance = vault.balance.checked_add(amount).ok_or(CustomError::Overflow)?;
        msg!("Deposited {} lamports safely, new balance: {}", amount, vault.balance);
        Ok(())
    }

    // Fixed: Use checked_sub to prevent underflow.
    // Educational: If subtraction would go below 0, reverts with Underflow error.
    pub fn withdraw(ctx: Context<DepositWithdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.balance = vault.balance.checked_sub(amount).ok_or(CustomError::Underflow)?;
        msg!("Withdrew {} lamports safely, new balance: {}", amount, vault.balance);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 8 + 32,
        seeds = [b"vault", user.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositWithdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub user: Signer<'info>,
}

#[account]
pub struct Vault {
    pub balance: u64,
    pub owner: Pubkey,
}

#[error_code]
pub enum CustomError {
    #[msg("Arithmetic overflow occurred")]
    Overflow,
    #[msg("Arithmetic underflow occurred")]
    Underflow,
}
