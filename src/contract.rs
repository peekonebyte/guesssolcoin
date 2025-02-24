use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("your_program_id");

#[program]
pub mod points_token {
    use super::*;

    // 初始化代币
    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        symbol: String,
        decimals: u8,
    ) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        token_info.name = name;
        token_info.symbol = symbol;
        token_info.decimals = decimals;
        token_info.authority = ctx.accounts.authority.key();
        Ok(())
    }

    // 铸造代币
    pub fn mint_token(
        ctx: Context<MintToken>,
        amount: u64,
    ) -> Result<()> {
        // 只有管理员可以铸造代币
        require!(
            ctx.accounts.token_info.authority == ctx.accounts.authority.key(),
            ErrorCode::Unauthorized
        );

        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    // 积分转换为代币
    pub fn convert_points(
        ctx: Context<ConvertPoints>,
        points_amount: u64,
    ) -> Result<()> {
        // 设置积分和代币的转换比率 (例如 10:1)
        let conversion_rate: u64 = 10;
        let token_amount = points_amount / conversion_rate;

        // 检查用户积分余额
        require!(
            ctx.accounts.user_points.amount >= points_amount,
            ErrorCode::InsufficientPoints
        );

        // 扣除积分
        ctx.accounts.user_points.amount -= points_amount;

        // 铸造对应数量的代币
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::mint_to(cpi_ctx, token_amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32 + 32 + 1)]
    pub token_info: Account<'info, TokenInfo>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub token_info: Account<'info, TokenInfo>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ConvertPoints<'info> {
    #[account(mut)]
    pub token_info: Account<'info, TokenInfo>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_points: Account<'info, UserPoints>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub authority: Pubkey,
}

#[account]
pub struct UserPoints {
    pub owner: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient points balance")]
    InsufficientPoints,
}
