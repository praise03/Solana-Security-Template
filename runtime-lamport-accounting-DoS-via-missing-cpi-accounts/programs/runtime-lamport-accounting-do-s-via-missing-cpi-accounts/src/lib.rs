use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("5nXgmC8gWLx9zA9zredHVJsV2ECz7BomWQf5uby8jRyA");


#[derive(Accounts)]
pub struct Initialize {}

/*
    =========================================================================
    DIRECT LAMPORT ACCESS CPI DOS — EDUCATIONAL EXAMPLE
    =========================================================================

    This program demonstrates a subtle Solana runtime rule:

    If you mutate lamports directly on an account, that account MUST be
    included in any subsequent CPI call within the same instruction,
    even if the CPI does not logically need it.

    Failing to do so causes a runtime error and a protocol-level DoS.
*/

#[program]
pub mod direct_lamport_cpi_dos {
    use super::*;

    /*
        ---------------------------------------------------------------------
        VULNERABLE INSTRUCTION
        ---------------------------------------------------------------------

        Pattern:
        1. Direct lamport transfer between program-owned accounts
        2. CPI that does NOT include all modified accounts

        Result:
        Runtime balance check fails → transaction aborts
    */
    pub fn vulnerable_settlement(ctx: Context<VulnerableSettlement>, amount: u64) -> Result<()> {
        let from = &ctx.accounts.vault_a;
        let to = &ctx.accounts.vault_b;

        /*
            STEP 1: DIRECT LAMPORT MANIPULATION

            This is legal as long as both accounts are program-owned.
        */
        **from.to_account_info().try_borrow_mut_lamports()? -= amount;
        **to.to_account_info().try_borrow_mut_lamports()? += amount;

        /*
            STEP 2: CPI THAT ONLY INVOLVES vault_b

            Problem:
            - vault_a lost lamports
            - vault_a is NOT part of this CPI
            - runtime lamport sum check during CPI sees imbalance
        */
        let cpi_accounts = Transfer {
            from: ctx.accounts.token_source.to_account_info(),
            to: ctx.accounts.token_dest.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );

        token::transfer(cpi_ctx, 1)?;

        /*
            The instruction never reaches here.
            The runtime aborts during the CPI.
        */
        Ok(())
    }

    /*
        ---------------------------------------------------------------------
        SAFE INSTRUCTION
        ---------------------------------------------------------------------

        Same logic, same lamport mutation.

        The only difference:
        All lamport-mutated accounts are passed into the CPI
        as remaining accounts.
    */
    pub fn safe_settlement(ctx: Context<SafeSettlement>, amount: u64) -> Result<()> {
        let from = &ctx.accounts.vault_a;
        let to = &ctx.accounts.vault_b;

        // Same direct lamport transfer
        **from.to_account_info().try_borrow_mut_lamports()? -= amount;
        **to.to_account_info().try_borrow_mut_lamports()? += amount;

        let cpi_accounts = Transfer {
            from: ctx.accounts.token_source.to_account_info(),
            to: ctx.accounts.token_dest.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        /*
            CRITICAL FIX:

            vault_a and vault_b are included in the CPI account list,
            even though the token program does not use them.
        */
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts)
            .with_remaining_accounts(
            vec![
                ctx.accounts.vault_a.to_account_info(),
                ctx.accounts.vault_b.to_account_info(),
            ]);

        token::transfer(cpi_ctx, 1)?;

        Ok(())
    }
}

/*
    -------------------------------------------------------------------------
    ACCOUNTS
    -------------------------------------------------------------------------
*/

#[derive(Accounts)]
pub struct VulnerableSettlement<'info> {
    #[account(mut)]
    pub vault_a: AccountInfo<'info>, // lamports debited

    #[account(mut)]
    pub vault_b: AccountInfo<'info>, // lamports credited

    #[account(mut)]
    pub token_source: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_dest: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SafeSettlement<'info> {
    #[account(mut)]
    pub vault_a: AccountInfo<'info>,

    #[account(mut)]
    pub vault_b: AccountInfo<'info>,

    #[account(mut)]
    pub token_source: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_dest: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
