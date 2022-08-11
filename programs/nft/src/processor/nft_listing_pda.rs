use {anchor_lang::prelude::*, anchor_spl::token};

pub fn create_nft_listing_pda(ctx: Context<CreateNftListing>) -> Result<()> {
    msg!("Set The Nft Listing PDA");

    msg!(
        "Nft Pda Address: {}",
        &ctx.accounts.nft_listing_account.key()
    );

    let nft_listing_account = &mut ctx.accounts.nft_listing_account;
    nft_listing_account.amount = 0;
    nft_listing_account.status = NftListingStatus::Closed;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateNftListing<'info> {
    #[account(mut)]
    pub mint: Account<'info, token::Mint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 82,
        seeds = [
            mint.key().as_ref(),
            b"_state",
        ],
        bump
    )]
    pub nft_listing_account: Account<'info, NftListingData>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct NftListingData {
    pub amount: u32,
    pub status: NftListingStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum NftListingStatus {
    Active,
    Closed,
}
