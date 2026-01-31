
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};

declare_id!("GQLLDZkQ9YRed6cU2vGfxGE6QmMEmX5t5hTksnCJ4eGK");

const TRUSTED_PROGRAM: &str = "Attacker111111111111111111111111111111111";

#[program]
pub mod unsafe_cpi_fixed {

    use super::*;

    pub fn call_external(ctx: Context<CallExternal>) -> Result<()> {
        // ✅ Validate CPI target
        require!(
            ctx.accounts.external_program.key().to_string() == TRUSTED_PROGRAM,
            ErrorCode::InvalidProgram
        );

        let ix = Instruction {
            program_id: ctx.accounts.external_program.key(),
            accounts: vec![],
            data: vec![],
        };

        invoke(&ix, &[ctx.accounts.user.to_account_info()])?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CallExternal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: validated by address check
    pub external_program: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid external program")]
    InvalidProgram,
}
