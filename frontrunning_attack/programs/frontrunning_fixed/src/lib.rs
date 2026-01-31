use anchor_lang::prelude::*;

declare_id!("7rVy6NkTb77NqzWrwKGJqMuzGKLtNJfB546gp7SY5jRD");

#[program]
pub mod frontrunning_fixed {
    use super::*;

    pub fn init_balance(ctx: Context<InitBalance>, amount: u64) -> Result<()> {
        ctx.accounts.balance.balance = amount;
        Ok(())
    }


    pub fn create_order(ctx: Context<CreateOrder>, price: u64, amount: u64) -> Result<()> {
        let order = &mut ctx.accounts.order;
        order.maker = ctx.accounts.maker.key();
        order.price = price;
        order.amount = amount;
        Ok(())
    }

    // ✅ Fixed: taker locks expected price
    pub fn fill_order(
        ctx: Context<FillOrder>,
        quantity: u64,
        expected_price: u64,
    ) -> Result<()> {
        let order = &mut ctx.accounts.order;

        require!(order.price == expected_price, ErrorCode::PriceChanged);
        require!(order.amount >= quantity, ErrorCode::NotEnoughLiquidity);

        let total_cost = order.price * quantity;

        ctx.accounts.taker_balance.balance -= total_cost;
        ctx.accounts.maker_balance.balance += total_cost;

        order.amount -= quantity;
        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
        ctx.accounts.order.price = new_price;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitBalance<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub balance: Account<'info, Balance>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct CreateOrder<'info> {
    #[account(init, payer = maker, space = 8 + 32 + 8 + 8)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub maker: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FillOrder<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub taker_balance: Account<'info, Balance>,
    #[account(mut)]
    pub maker_balance: Account<'info, Balance>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut, has_one = maker)]
    pub order: Account<'info, Order>,
    pub maker: Signer<'info>,
}

#[account]
pub struct Order {
    pub maker: Pubkey,
    pub price: u64,
    pub amount: u64,
}

#[account]
pub struct Balance {
    pub balance: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Order price changed")]
    PriceChanged,
    #[msg("Not enough liquidity")]
    NotEnoughLiquidity,
}

