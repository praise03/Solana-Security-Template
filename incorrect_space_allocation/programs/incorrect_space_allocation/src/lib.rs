use anchor_lang::prelude::*;

declare_id!("Bubm1PXV2WEFR1xzXmRUwa7hpCJJgjG42kJT8GtXoSyE");

/*
    =========================================================================
    INCORRECT ACCOUNT SPACE ALLOCATION — EDUCATIONAL EXAMPLE
    =========================================================================

    This program demonstrates a common Anchor mistake:

    - Manually calculating PDA account space
    - Forgetting discriminator padding, alignment, or future changes
    - Causing account truncation, runtime panics, or protocol-level DoS

    Anchor provides safer alternatives that many programs still ignore.
*/

#[program]
pub mod incorrect_account_space {
    use super::*;

    /*
        ---------------------------------------------------------------------
        VULNERABLE INITIALIZATION
        ---------------------------------------------------------------------

        The developer manually calculates space for the PDA.
        The calculation is WRONG.
    */
    pub fn init_user_vulnerable(ctx: Context<InitUserVulnerable>) -> Result<()> {
        let user = &mut ctx.accounts.user_account;

        user.owner = ctx.accounts.authority.key();
        user.balance = 0;
        user.bump = ctx.bumps.user_account;

        /*
            This may appear to work initially.

            Problems appear when:
            - Fields are added or reordered
            - Anchor serialization layout changes
            - The account grows beyond allocated space

            Result:
            - Account data overwrite
            - Instruction panics
            - Permanent account bricking
        */
        Ok(())
    }

    /*
        ---------------------------------------------------------------------
        SAFE INITIALIZATION
        ---------------------------------------------------------------------

        Anchor derives space automatically from the struct.
        This removes human error entirely.
    */
    pub fn init_user_safe(ctx: Context<InitUserSafe>) -> Result<()> {
        let user = &mut ctx.accounts.user_account;

        user.owner = ctx.accounts.authority.key();
        user.balance = 0;
        user.bump = ctx.bumps.user_account;

        Ok(())
    }
}

/*
    -------------------------------------------------------------------------
    ACCOUNTS — VULNERABLE VERSION
    -------------------------------------------------------------------------
*/

#[derive(Accounts)]
pub struct InitUserVulnerable<'info> {
    #[account(
        init,
        payer = authority,
        /*
            ❌ MANUAL SPACE CALCULATION (WRONG)

            Mistakes commonly made here:
            - Forgetting the 8-byte discriminator
            - Miscounting field sizes
            - Ignoring future struct changes

            This example under-allocates space.
        */
        space = 32 + 8 + 1, // incorrect
        seeds = [b"user", authority.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/*
    -------------------------------------------------------------------------
    ACCOUNTS — SAFE VERSION
    -------------------------------------------------------------------------
*/

#[derive(Accounts)]
pub struct InitUserSafe<'info> {
    #[account(
        init,
        payer = authority,
        /*
            ✅ DERIVED SPACE

            Anchor computes the correct size:
            - Discriminator
            - Field layout
            - Alignment
            - Future-proof against struct edits
        */
        space = UserAccount::INIT_SPACE,
        seeds = [b"user", authority.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/*
    -------------------------------------------------------------------------
    ACCOUNT STATE
    -------------------------------------------------------------------------
*/

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

