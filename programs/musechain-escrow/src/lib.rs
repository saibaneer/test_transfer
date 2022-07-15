use anchor_lang::prelude::*;
use anchor_spl::token::{CloseAccount, Token, TokenAccount, Transfer};

declare_id!("3tB4rGwEj3B8YjozEBTHL5f8Tzubq9kiYJP4E6nyKQDX");

#[program]
pub mod musechain_escrow {
    use anchor_lang::solana_program::lamports;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, mint_address: Pubkey) -> Result<()> {
        let lock_account = &mut ctx.accounts.lock_account;
        lock_account.owner = *ctx.accounts.owner.key;
        lock_account.authority = ctx.accounts.owner.key().clone();
        lock_account.bump = *ctx.bumps.get("owner").unwrap();
        lock_account.mint_address = mint_address;
        lock_account.escrow_bump = *ctx.bumps.get("escrow").unwrap();
        lock_account.escrow_pda = *ctx.accounts.lock_escrow_account.to_account_info().key;
        Ok(())
    }

    pub fn list_nft(ctx: Context<ListNFT>, price: u64) -> Result<()> {
        let lock_account = &mut ctx.accounts.lock_account;
        let lock_escrow_account = &mut ctx.accounts.lock_escrow_account;
        lock_escrow_account.price = price;
        lock_escrow_account.owner = ctx.accounts.owner.key().clone();
        lock_escrow_account.mint = ctx.accounts.mint_address.key().clone();

        let transfer_instruction = Transfer {
            from: ctx.accounts.owner.to_account_info(),
            to: ctx.accounts.lock_escrow_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        // Create the Context for our Transfer request
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);

        // Execute anchor's helper function to transfer tokens
        anchor_spl::token::transfer(cpi_ctx, 1)?;
        Ok(())
    }

    pub fn buy(ctx: Context<Buy>, lamports: u64, mint_address: Pubkey) -> Result<()> {
        let lock_account = &mut ctx.accounts.lock_account;
        let lock_escrow_account = &mut ctx.accounts.lock_escrow_account;

        **lock_escrow_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= lamports;
        **ctx
            .accounts
            .buyer
            .to_account_info()
            .try_borrow_mut_lamports()? += lamports;

        let transfer_instruction = Transfer {
            from: ctx.accounts.lock_escrow_account.to_account_info(),
            to: ctx.accounts.buyer.to_account_info(),
            authority: lock_account.to_account_info(),
        };

        let inner = vec![b"escrow", ctx.accounts.mint_address.key.as_ref()];
        let outer = vec![inner.as_slice()];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
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
    #[account(init, payer = owner, space=8+32+32+32+1+1+32, seeds=[b"owner", owner.key().as_ref()], bump)]
    pub lock_account: Account<'info, LockAccount>,

    #[account(init, payer = owner, space=8+1+32+32, seeds=[b"escrow", mint_address.key().as_ref()], bump)]
    pub lock_escrow_account: Account<'info, LockEscrowAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub mint_address: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListNFT<'info> {
    #[account(has_one = owner)]
    pub lock_account: Account<'info, LockAccount>,

    /// CHECK: There is no reason I can think of as to why this is unsafe.
    #[account(signer)]
    pub owner: AccountInfo<'info>,

    #[account(mut)]
    pub mint_address: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    #[account( mut, constraint = lock_account.escrow_pda == *lock_escrow_account.to_account_info().key)]
    pub lock_escrow_account: Account<'info, LockEscrowAccount>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub lock_account: Account<'info, LockAccount>,

    #[account(mut, signer)]
    pub buyer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,

    #[account(mut, constraint = lock_account.escrow_pda == *lock_escrow_account.to_account_info().key)]
    //How do I add a constraint where the lamports must equal price?
    //#[account( mut)]
    pub lock_escrow_account: Account<'info, LockEscrowAccount>,

    #[account(mut)]
    pub mint_address: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct LockAccount {
    pub owner: Pubkey,        //32 bytes
    pub authority: Pubkey,    //32 bytes
    pub mint_address: Pubkey, //32 bytes
    pub bump: u8,             // 32 bytes
    pub escrow_bump: u8,      //32 bytes
    pub escrow_pda: Pubkey,   //32 bytes
}

#[account]
pub struct LockEscrowAccount {
    pub price: u64,
    pub owner: Pubkey,
    pub mint: Pubkey,
}
