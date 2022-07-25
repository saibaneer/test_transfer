use anchor_lang::prelude::*;
use anchor_spl::token::{CloseAccount, Token, TokenAccount, Transfer, Mint};

declare_id!("3tB4rGwEj3B8YjozEBTHL5f8Tzubq9kiYJP4E6nyKQDX");

#[program]
pub mod musechain_escrow {
    use anchor_lang::solana_program::lamports;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let lock_account = &mut ctx.accounts.lock_account;
        lock_account.owner = ctx.accounts.owner.key();
        lock_account.mint_address = ctx.accounts.mint_address.key();
        Ok(())
    }

    pub fn list_nft(ctx: Context<ListNFT>, lock_account_bump: u8, escrow_token_bump: u8,  price: u64) -> Result<()> {
        let lock_account = &mut ctx.accounts.lock_account;
        lock_account.price = price;
        let mint = ctx.accounts.mint_address.key();

        let bump_vector = lock_account_bump.to_le_bytes();
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let inner = vec![
            b"owner".as_ref(),
            ctx.accounts.owner.key.as_ref(),
            mint.as_ref(),
            bump_vector.as_ref(),
        ];

        let outer = vec![inner.as_slice()];
        let transfer_instruction = Transfer {
            from: ctx.accounts.nft_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        // Create the Context for our Transfer request
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_instruction, outer.as_slice());

        // Execute anchor's helper function to transfer tokens
        anchor_spl::token::transfer(cpi_ctx, 1)?;
        Ok(())
    }

    pub fn buy(ctx: Context<Buy>, lock_account_bump: u8, escrow_token_bump: u8) -> Result<()> {
        let lock_account = &mut ctx.accounts.lock_account;
        if ctx.accounts.seller.key() != lock_account.owner {
            return Err(error!(ErrorCode::InvalidSeller))
        }
        let mint = ctx.accounts.mint_address.key();
        if ctx.accounts.seller.key() != ctx.accounts.buyer.key() {
            let first_ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.seller.key(),
                lock_account.price,
            );
            anchor_lang::solana_program::program::invoke(
                &first_ix,
                &[
                    ctx.accounts.buyer.to_account_info(),
                    ctx.accounts.seller.to_account_info(),
                ],
            );
        }
        let bump_vector = lock_account_bump.to_le_bytes();
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let inner = vec![
            b"owner".as_ref(),
            ctx.accounts.seller.key.as_ref(),
            mint.as_ref(),
            bump_vector.as_ref(),
        ];

        let outer = vec![inner.as_slice()];
        let transfer_instruction = Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: lock_account.to_account_info(),
        };  

        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            transfer_instruction,
            outer.as_slice(),
        );

        // Execute anchor's helper function to transfer tokens
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space=8+32+32+32+1+1+32, seeds=[b"owner", owner.key().as_ref(), mint_address.key().as_ref()], bump)]
    pub lock_account: Account<'info, LockAccount>,
    #[account(init, payer = owner, seeds = [b"token", owner.key().as_ref(), mint_address.key().as_ref()], bump, token::mint = mint_address, token::authority = lock_account)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub mint_address: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(lock_account_bump: u8, escrow_wallet_bump: u8)]
pub struct ListNFT<'info> {
    #[account(mut, seeds = [b"owner", owner.key().as_ref(), mint_address.key().as_ref()] , bump = lock_account_bump, has_one = owner)]
    pub lock_account: Account<'info, LockAccount>,
    #[account(mut, seeds = [b"token", owner.key().as_ref(), mint_address.key().as_ref()], bump = escrow_wallet_bump)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint_address: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(lock_account_bump: u8, escrow_token_bump: u8)]
pub struct Buy<'info> {
    #[account(mut, seeds = [b"owner", seller.key().as_ref(), mint_address.key().as_ref()] , bump = lock_account_bump)]
    pub lock_account: Account<'info, LockAccount>,
    #[account(mut, seeds = [b"token", seller.key().as_ref(), mint_address.key().as_ref()], bump = escrow_token_bump)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    ///CHECK:
    pub seller: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint_address: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
pub struct LockAccount {
    pub owner: Pubkey,        //32 bytes
    pub mint_address: Pubkey, //32 bytes
    pub price: u64, // 8 bytes
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid seller account, please send the right account again")]
    InvalidSeller
}