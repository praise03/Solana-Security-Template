use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{
    PriceUpdateV2,
    get_price_no_older_than,
};

declare_id!("E8i883WGjhp5o2Cybz5PkmgJPtBJeoXPAxQehDUoyE2R");


/*
    =========================================================================
    NON-CANONICAL PYTH PRICE FEED EXAMPLE
    =========================================================================

    This program demonstrates a subtle but real oracle vulnerability when
    consuming Pyth PriceUpdateV2 accounts.

    Key background facts (often misunderstood):

    - PriceUpdateV2 accounts are user-provided.
    - They are NOT canonical oracle state.
    - Anyone can create one by calling the Pyth receiver program.
    - Validation APIs only prove the price was valid at some point in time.
*/

#[program]
pub mod stale_oracle_replay {
    use super::*;

    /*
        ---------------------------------------------------------------------
        VULNERABLE INSTRUCTION
        ---------------------------------------------------------------------

        This instruction mirrors code used to fetch prices from Pyth oracles

        It does the following correctly:
        - Validates the price feed ID
        - Enforces a max_age window
        - Uses official Pyth SDK helpers

        It still has a serious flaw.
    */
    pub fn consume_price_vulnerable(
        ctx: Context<ConsumePriceVulnerable>,
        expected_feed_id: [u8; 32],
        max_age_seconds: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;

        /*
            The PriceUpdateV2 account is passed in by the user.
        */
        let price_update: &PriceUpdateV2 =
            &ctx.accounts.price_update.load()?;

        /*
            This helper checks two things:
            1. The feed_id matches
            2. The price is not older than max_age_seconds

            What it DOES NOT check:
            - Whether this price is newer than previously used prices
            - Whether another more recent price has already been processed
        */
        let price = get_price_no_older_than(
            price_update,
            clock.unix_timestamp,
            max_age_seconds,
        )
        .map_err(|_| error!(ErrorCode::InvalidOrStalePrice))?;

        require!(
            price_update.feed_id == expected_feed_id,
            ErrorCode::InvalidFeed
        );

        /*
            At this point, many developers believe they are safe.

            They are not.

            The protocol now acts on the price assuming it represents
            "the current oracle price", which is an invalid assumption.
        */
        msg!(
            "VULNERABLE CONSUME: price={}, publish_time={}",
            price.price,
            price.publish_time
        );

        /*
            Any state change here (liquidation, mint, borrow, settlement)
            is vulnerable to time rollback if a newer price was already
            processed in a prior transaction.
        */

        Ok(())
    }

    /*
        ---------------------------------------------------------------------
        SAFE INSTRUCTION
        ---------------------------------------------------------------------

        This version fixes the vulnerability by introducing protocol-level
        ordering guarantees.
    */
    pub fn consume_price_safe(
        ctx: Context<ConsumePriceSafe>,
        expected_feed_id: [u8; 32],
        max_age_seconds: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let oracle_state = &mut ctx.accounts.oracle_state;

        let price_update: &PriceUpdateV2 =
            &ctx.accounts.price_update.load()?;

        let price = get_price_no_older_than(
            price_update,
            clock.unix_timestamp,
            max_age_seconds,
        )
        .map_err(|_| error!(ErrorCode::InvalidOrStalePrice))?;

        require!(
            price_update.feed_id == expected_feed_id,
            ErrorCode::InvalidFeed
        );

        /*
            CRITICAL DEFENSE:

            We enforce that publish_time must be strictly increasing
            relative to what this protocol has already accepted.This closes the replay and rollback vector entirely.

            This defense prevents attackers from tricking your program with old prices. Without it, 
            someone could submit a price that’s technically valid but older than the one your protocol 
            already used, effectively rolling back the price. By requiring that each new price has a timestamp 
            later than the last one you accepted, the program ensures it only ever moves forward in time, 
            blocking replay or “stale” price attacks.
        */
        require!(
            price.publish_time > oracle_state.last_publish_time,
            ErrorCode::OutOfOrderPrice
        );

        /*
            Update protocol state before performing any sensitive logic.
            This ensures reentrancy and CPI edge cases do not bypass ordering.
        */
        oracle_state.last_publish_time = price.publish_time;

        msg!(
            "SAFE CONSUME: price={}, publish_time={}",
            price.price,
            price.publish_time
        );

        Ok(())
    }
}

/*
    -------------------------------------------------------------------------
    ACCOUNTS
    -------------------------------------------------------------------------
*/

#[derive(Accounts)]
pub struct ConsumePriceVulnerable<'info> {
    /*
        CHECK:

        This is exactly how many protocols accept PriceUpdateV2.

        The address is unconstrained because developers assume the
        Pyth program guarantees correctness.

        That assumption is false.
    */
    pub price_update: AccountLoader<'info, PriceUpdateV2>,
}

#[derive(Accounts)]
pub struct ConsumePriceSafe<'info> {
    pub price_update: AccountLoader<'info, PriceUpdateV2>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + OracleState::SIZE
    )]
    pub oracle_state: Account<'info, OracleState>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}


#[account]
pub struct OracleState {
    /*
        Stores the most recent publish_time the protocol has accepted.

        This value is the missing invariant in the vulnerable version.
    */
    pub last_publish_time: i64,
}

impl OracleState {
    pub const SIZE: usize = 8;
}


#[error_code]
pub enum ErrorCode {
    #[msg("Price feed ID does not match expected feed")]
    InvalidFeed,

    #[msg("Price is invalid or older than max_age")]
    InvalidOrStalePrice,

    #[msg("Price publish_time is older than a previously accepted price")]
    OutOfOrderPrice,
}
