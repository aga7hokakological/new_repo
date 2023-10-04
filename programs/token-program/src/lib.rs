use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer, Token, Mint, Burn, TokenAccount};

declare_id!("5EyzFuQFafP4Mv1JPqi9EFzaEJUXCRBb5GPvF5YByuU6");

#[program]
pub mod token_program {
    use super::*;

    pub fn initialize_dex(
        ctx: Context<InitializeDex>, 
    ) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex;

        dex_account.is_initialized = true;
        dex_account.authority = ctx.accounts.authority.key();
        dex_account.token0 = ctx.accounts.mint_token0.key();
        dex_account.token1 = ctx.accounts.mint_token1.key();
        dex_account.lp_token = ctx.accounts.mint_lp.key();
        dex_account.k = 0;

        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<LiquidityOperations>,
        token0_amount: u64,
        token1_amount: u64
    ) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex;

        let liquidity = (token0_amount.checked_mul(token1_amount)).unwrap();

        let old_k = dex_account.k;

        dex_account.lp_amount = dex_account.lp_amount.checked_add(liquidity).unwrap();
        dex_account.token0_amount = dex_account.token0_amount.checked_add(token0_amount).unwrap();
        dex_account.token1_amount = dex_account.token1_amount.checked_add(token1_amount).unwrap();
        dex_account.k = (
            dex_account.token0_amount.
            checked_mul(dex_account.token1_amount)
        ).unwrap();

        assert!(dex_account.k >= old_k);


        // Transfer token0 from user ATA to dex ATA
        let cpi_accounts_token0 = Transfer {
            from: ctx.accounts.user_token0.to_account_info(),
            to: ctx.accounts.account_token0.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program_token0 = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_token0 = CpiContext::new(cpi_program_token0, cpi_accounts_token0);
        token::transfer(cpi_ctx_token0, token0_amount)?;

        // Transfer token1 from user ATA to dex ATA
        let cpi_accounts_token1 = Transfer {
            from: ctx.accounts.user_token1.to_account_info(),
            to: ctx.accounts.account_token1.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program_token1 = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_token1 = CpiContext::new(cpi_program_token1, cpi_accounts_token1);
        token::transfer(cpi_ctx_token1, token1_amount)?;

        // Mint LP tokens to user ATA
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint_lp.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, liquidity)?;

        Ok(())
    }

    pub fn remove_liquidity(ctx: Context<LiquidityOperations>, amount: u64) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex;

        let old_k = dex_account.k;

        let amount0 = (dex_account.lp_amount.checked_mul(dex_account.token0_amount)).unwrap()
                                            .checked_div(dex_account.lp_amount).unwrap();
        let amount1 = (dex_account.lp_amount.checked_mul(dex_account.token1_amount)).unwrap()
                                            .checked_div(dex_account.lp_amount).unwrap();

        dex_account.lp_amount = dex_account.lp_amount.checked_sub(amount).unwrap();
        dex_account.token0_amount = dex_account.token0_amount.checked_sub(amount0).unwrap();
        dex_account.token1_amount = dex_account.token1_amount.checked_sub(amount1).unwrap();

        dex_account.k = (
            dex_account.token0_amount.
            checked_mul(dex_account.token1_amount)
        ).unwrap();

        assert!(dex_account.k >= old_k);

        let cpi_accounts_lp = Burn {
            mint: ctx.accounts.token_program.to_account_info(),
            from: ctx.accounts.user.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program_lp = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_lp = CpiContext::new(cpi_program_lp, cpi_accounts_lp);
        token::burn(cpi_ctx_lp, amount);

        // Transfer token0 from user ATA to dex ATA
        let cpi_accounts_token0 = Transfer {
            from: ctx.accounts.account_token0.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program_token0 = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_token0 = CpiContext::new(cpi_program_token0, cpi_accounts_token0);
        token::transfer(cpi_ctx_token0, amount0)?;

        // Transfer token1 from user ATA to dex ATA
        let cpi_accounts_token1 = Transfer {
            from: ctx.accounts.account_token1.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program_token1 = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_token1 = CpiContext::new(cpi_program_token1, cpi_accounts_token1);
        token::transfer(cpi_ctx_token1, amount1)?;

        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, token_in: Pubkey, token_amt_in: u64, token_amt_out: u64) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex;

        let amt0;
        let amt1;

        assert!(
            token_in.key() == dex_account.token0.key() || 
            token_in.key() == dex_account.token1.key()
        );

        let reserve0 = dex_account.token0_amount;
        let reserve1 = dex_account.token1_amount;

        if token_in.key() == dex_account.token0.key() {
            amt0 = token_amt_in;
            amt1 = 0;

            dex_account.token0_amount = dex_account.token0_amount.
                                                    checked_add(token_amt_in).unwrap();

            // Transfer token0 from user ATA to dex ATA
            let cpi_accounts_token0 = Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.account_token0.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi_program_token0 = ctx.accounts.token_program.to_account_info();
            let cpi_ctx_token0 = CpiContext::new(cpi_program_token0, cpi_accounts_token0);
            token::transfer(cpi_ctx_token0, token_amt_in)?;
        } else if token_in.key() == dex_account.token1.key() {
            amt0 = 0;
            amt1 = token_amt_in;

            dex_account.token0_amount = dex_account.token0_amount.
                                                    checked_add(token_amt_in).unwrap();

            // Transfer token1 from user ATA to dex ATA
            let cpi_accounts_token1 = Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.account_token1.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi_program_token1 = ctx.accounts.token_program.to_account_info();
            let cpi_ctx_token1 = CpiContext::new(cpi_program_token1, cpi_accounts_token1);
            token::transfer(cpi_ctx_token1, token_amt_in)?;
        } else {
            msg!("{:?}", ErrorCode::WrongInputToken);
        }

        let balance0 = dex_account.token0_amount;
        let balance1 = dex_account.token1_amount;


        assert!(
            balance0.checked_mul(balance1) >= reserve0.checked_mul(reserve1)
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeDex<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 1024,
        seeds = [b"dex".as_ref(), authority.key().as_ref()],
        bump
    )]
    pub dex: Account<'info, Dex>,
    pub mint_token0: Account<'info, Mint>,
    pub mint_token1: Account<'info, Mint>,
    pub mint_lp: Account<'info, Mint>,
    pub account_token0: Account<'info, TokenAccount>,
    pub account_token1: Account<'info, TokenAccount>,
    pub account_lp: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    // pub token1_program: Program<'info, Token>,
    // pub lp_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    /// CHECK: We are not reading writing from user acc
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub dex: Account<'info, Dex>,
    pub mint_token0: Account<'info, Mint>,
    pub mint_token1: Account<'info, Mint>,
    pub mint_lp: Account<'info, Mint>,
    pub account_token0: Account<'info, TokenAccount>,
    pub account_token1: Account<'info, TokenAccount>,
    pub account_lp: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    // pub token1_program: Program<'info, Token>,
    // pub lp_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct LiquidityOperations<'info> { 
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: We are not reading writing from user acc
    #[account(mut)]
    pub authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub dex: Account<'info, Dex>,
    pub mint_token0: Account<'info, Mint>,
    pub mint_token1: Account<'info, Mint>,
    pub mint_lp: Account<'info, Mint>,
    pub user_token0: Account<'info, TokenAccount>,
    pub user_token1: Account<'info, TokenAccount>,
    pub user_lp: Account<'info, TokenAccount>,
    pub account_token0: Account<'info, TokenAccount>,
    pub account_token1: Account<'info, TokenAccount>,
    pub account_lp: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    // pub token1_program: Program<'info, Token>,
    // pub lp_program: Program<'info, Token>,
    // pub system_program: Program<'info, System>,
}

#[account]
pub struct Dex {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub token0: Pubkey,
    pub token1: Pubkey,
    pub lp_token: Pubkey,
    pub token0_amount: u64,
    pub token1_amount: u64,
    pub lp_amount: u64,
    pub k: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Wrong input token")]
    WrongInputToken,
}