use anchor_lang::prelude::*;

declare_id!("EAFqQpSvFA6iT1aw6VBu4HzxjaCNajVn73VRRETZiMaP");


#[program]
pub mod seed_collision {
    use super::*;

    /*
        VULNERABLE INSTRUCTION

        This instruction derives a PDA using seeds that are NOT
        sufficiently domain-separated.

        The seed scheme allows multiple logical entities to map
        to the same PDA address, creating a seed collision.

        In real protocols, this can lead to:
        - account overwrites
        - unauthorized access
        - state corruption
    */
    pub fn initialize_vulnerable(
        ctx: Context<VulnerableCtx>,
        user_id: u64,
    ) -> Result<()> {
        // Store user-controlled data
        ctx.accounts.state.owner = ctx.accounts.user.key();
        ctx.accounts.state.user_id = user_id;

        Ok(())
    }

    /*
        SAFE INSTRUCTION

        This instruction uses explicit domain separation in seeds.
        Each logical account type has a unique, fixed prefix.

        This guarantees that unrelated accounts cannot collide,
        even if user-controlled inputs are identical.
    */
    pub fn initialize_safe(
        ctx: Context<SafeCtx>,
        user_id: u64,
    ) -> Result<()> {
        ctx.accounts.state.owner = ctx.accounts.user.key();
        ctx.accounts.state.user_id = user_id;

        Ok(())
    }
}

//
// --------------------
// Vulnerable Accounts
// --------------------
//

#[derive(Accounts)]
#[instruction(user_id: u64)]
pub struct VulnerableCtx<'info> {
    /*
        BUG:
        The PDA is derived using only user-controlled input.

        If another instruction elsewhere uses the same seed pattern
        (e.g. for a different account type), the derived address
        will be identical.

        This is a seed collision.
    */
    #[account(
        init,
        payer = user,
        space = 8 + State::INIT_SPACE,
        seeds = [user_id.to_le_bytes().as_ref()],
        bump
    )]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

//
// --------------------
// Safe Accounts
// --------------------
//

#[derive(Accounts)]
#[instruction(user_id: u64)]
pub struct SafeCtx<'info> {
    /*
        FIX:
        Add a fixed, hardcoded prefix to the seeds.

        This provides domain separation and ensures
        this PDA cannot collide with other account types.
    */
    #[account(
        init,
        payer = user,
        space = 8 + State::INIT_SPACE,
        seeds = [
            b"user_state",
            user_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

//
// --------------------
// State
// --------------------
//

#[account]
pub struct State {
    pub owner: Pubkey,
    pub user_id: u64,
}

impl State {
    pub const INIT_SPACE: usize = 32 + 8;
}

