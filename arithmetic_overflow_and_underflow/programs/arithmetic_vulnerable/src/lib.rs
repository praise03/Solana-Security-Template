use anchor_lang::prelude::*;

declare_id!("2iiq3EH648pWmcLZQk3WfQHXCLeaLQwfNriwPTX64iBz");

#[program]
pub mod arithmetic_vulnerable {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.balance = 0;
        vault.owner = *ctx.accounts.user.key;
        msg!("Vault initialized with balance 0");
        Ok(())
    }

    // Educational: Instruction to deposit lamports into a vault account.
    // Vulnerable to overflow: raw addition can wrap around u64 max.
    pub fn deposit(ctx: Context<DepositWithdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.balance += amount;  // ← Vulnerable: no overflow check, can wrap to 0 or low value
        msg!("Deposited {} lamports, new balance: {}", amount, vault.balance);
        Ok(())
    }

    // Educational: Instruction to withdraw lamports from the vault.
    // Vulnerable to underflow: raw subtraction can wrap to huge value if amount > balance.
    pub fn withdraw(ctx: Context<DepositWithdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.balance -= amount;  // ← Vulnerable: no underflow check, can wrap to max u64
        msg!("Withdrew {} lamports, new balance: {}", amount, vault.balance);
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
