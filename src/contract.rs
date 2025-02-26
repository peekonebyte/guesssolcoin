use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("your_program_id");

#[program]
pub mod points_token {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        symbol: String,
        decimals: u8,
        default_conversion_rate: u64,
    ) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        token_info.name = name;
        token_info.symbol = symbol;
        token_info.decimals = decimals;
        token_info.authority = ctx.accounts.authority.key();
        token_info.default_conversion_rate = default_conversion_rate;
        Ok(())
    }

    pub fn mint_token(
        ctx: Context<MintToken>,
        amount: u64,
    ) -> Result<()> {
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

    pub fn update_default_conversion_rate(
        ctx: Context<UpdateDefaultRate>,
        new_rate: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.token_info.authority == ctx.accounts.authority.key(),
            ErrorCode::Unauthorized
        );
        
        require!(new_rate > 0, ErrorCode::InvalidConversionRate);
        
        ctx.accounts.token_info.default_conversion_rate = new_rate;
        Ok(())
    }
    
    pub fn set_custom_conversion_rate(
        ctx: Context<SetCustomRate>,
        custom_rate: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.token_info.authority == ctx.accounts.authority.key(),
            ErrorCode::Unauthorized
        );
        
        require!(custom_rate > 0, ErrorCode::InvalidConversionRate);
        
        let custom_rate_account = &mut ctx.accounts.custom_rate;
        custom_rate_account.user = ctx.accounts.user_account.key();
        custom_rate_account.conversion_rate = custom_rate;
        Ok(())
    }

    pub fn convert_points(
        ctx: Context<ConvertPoints>,
        points_amount: u64,
    ) -> Result<()> {
        let conversion_rate = if ctx.accounts.custom_rate.is_some() {
            let custom_rate = ctx.accounts.custom_rate.as_ref().unwrap();
            require!(
                custom_rate.user == ctx.accounts.user_points.owner,
                ErrorCode::InvalidCustomRateAccount
            );
            custom_rate.conversion_rate
        } else {
            ctx.accounts.token_info.default_conversion_rate
        };
        
        let token_amount = points_amount / conversion_rate;

        require!(
            ctx.accounts.user_points.amount >= points_amount,
            ErrorCode::InsufficientPoints
        );

        ctx.accounts.user_points.amount -= points_amount;

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

    pub fn batch_set_custom_rates(
        ctx: Context<BatchSetCustomRates>,
        user_rates: Vec<(Pubkey, u64)>,
    ) -> Result<()> {
        require!(
            ctx.accounts.token_info.authority == ctx.accounts.authority.key(),
            ErrorCode::Unauthorized
        );
        
        require!(
            user_rates.len() <= 10,
            ErrorCode::BatchTooLarge
        );
        
        ctx.accounts.batch_rates.authority = ctx.accounts.authority.key();
        ctx.accounts.batch_rates.user_rates = user_rates
            .into_iter()
            .map(|(user, rate)| {
                require!(rate > 0, ErrorCode::InvalidConversionRate);
                UserRate { user, rate }
            })
            .collect();
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32 + 32 + 1 + 8)]
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
pub struct UpdateDefaultRate<'info> {
    #[account(mut)]
    pub token_info: Account<'info, TokenInfo>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetCustomRate<'info> {
    pub token_info: Account<'info, TokenInfo>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + 32 + 8,
        seeds = [b"custom_rate", user_account.key().as_ref()],
        bump
    )]
    pub custom_rate: Account<'info, CustomConversionRate>,
    /// CHECK: 这是要设置专属兑换比例的用户账户
    pub user_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchSetCustomRates<'info> {
    pub token_info: Account<'info, TokenInfo>,
    #[account(
        init_if_needed, 
        payer = authority, 
        space = 8 + 32 + 4 + (10 * (32 + 8))
    )]
    pub batch_rates: Account<'info, BatchConversionRates>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
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
    /// 用户的专属兑换比例账户（可选）
    #[account(
        mut,
        seeds = [b"custom_rate", user_points.owner.as_ref()],
        bump,
        constraint = custom_rate.user == user_points.owner @ ErrorCode::InvalidCustomRateAccount,
        constraint = custom_rate.is_initialized @ ErrorCode::CustomRateNotInitialized,
    )]
    pub custom_rate: Option<Account<'info, CustomConversionRate>>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub authority: Pubkey,
    pub default_conversion_rate: u64,
}

#[account]
pub struct UserPoints {
    pub owner: Pubkey,
    pub amount: u64,
}

#[account]
#[derive(Default)]
pub struct CustomConversionRate {
    pub user: Pubkey,
    pub conversion_rate: u64,
    pub is_initialized: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserRate {
    pub user: Pubkey,
    pub rate: u64,
}

#[account]
pub struct BatchConversionRates {
    pub authority: Pubkey,
    pub user_rates: Vec<UserRate>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient points balance")]
    InsufficientPoints,
    #[msg("Invalid conversion rate")]
    InvalidConversionRate,
    #[msg("Invalid custom rate account")]
    InvalidCustomRateAccount,
    #[msg("Custom rate account not initialized")]
    CustomRateNotInitialized,
    #[msg("Batch size too large")]
    BatchTooLarge,
}
