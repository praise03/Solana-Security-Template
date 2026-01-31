use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    program::invoke,
};

declare_id!("CrzaDexxpR8ARo2rPUiG9ExLmPXScQVoFstHquQshFv5");

#[program]
pub mod unsafe_cpi_vulnerable {

    use super::*;

    pub fn call_external(ctx: Context<CallExternal>) -> Result<()> {
        // Manually encoded Anchor discriminator for: global::withdraw
        // (attacker program defines this)
        let discriminator: [u8; 8] = [0xe4, 0x9a, 0xc8, 0x8d, 0xe2, 0xb7, 0x94, 0x19];

        let ix = Instruction {
            program_id: ctx.accounts.external_program.key(),
            accounts: vec![
                AccountMeta::new(ctx.accounts.user.key(), true), // victim signer
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
            ],
            data: discriminator.to_vec(),
        };

        invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}

// #[derive(Accounts)]
// pub struct CallExternal<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,
//     /// CHECK: unchecked external program
//     pub external_program: AccountInfo<'info>,
// }

#[derive(Accounts)]
pub struct CallExternal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: unchecked external program (vulnerability)
    pub external_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

