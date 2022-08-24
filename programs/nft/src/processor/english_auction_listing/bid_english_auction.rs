use {
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{associated_token, token},
};

use crate::{
    error::ErrorCode,
    processor::english_auction_listing::utils::{
        create_english_auction_bid_pda::*, create_english_auction_listing_pda::*,
    },
    utils::create_nft_listing_pda::*,
    validate::{check_active_listing::*, check_listing_is_active::*, check_nft_owner::*},
};

pub fn bid_english_auction_fn(
    ctx: Context<BidEnglishAuction>,
    bid_price_lamports: u64,
) -> Result<()> {
    msg!("Bidding process started");

    let nft_listing = &ctx.accounts.nft_listing_account.to_account_info();

    let nft_listing_account = &mut ctx.accounts.nft_listing_account;

    let listing_account = &mut ctx.accounts.listing_account;
    let bid_account = &mut ctx.accounts.bid_account;

    // check the bidder is the same as the bid account Pda creator
    if bid_account.key() != ctx.accounts.bid_account_vault.key() {
        return Err(ErrorCode::InvalidData.into());
    }

    if bid_account.bidder.key() != ctx.accounts.bidder.key() {
        return Err(ErrorCode::BidderInvalidData.into());
    }

    check_listing_is_active(
        &ctx.program_id,
        &listing_account.mint,
        listing_account.is_active,
        &nft_listing_account,
    )?;

    check_nft_owner(
        &&listing_account.seller.clone(),
        &ctx.accounts.seller_token,
        nft_listing_account,
    )?;

    check_active_listing(
        listing_account.start_date,
        listing_account.end_date,
        listing_account.close_date,
        listing_account.starting_price_lamports,
        listing_account.sold,
        &nft_listing,
        &ctx.accounts.seller_token,
    )?;

    // sum up the total lamports that were deposited
    let mut bid_account_lamports: u64 = bid_price_lamports;
    if bid_account.bid_price_lamports.is_some() {
        bid_account_lamports = bid_account.bid_price_lamports.unwrap() + bid_price_lamports;
    }

    // check if the bid is higher than starting price
    if listing_account.starting_price_lamports > bid_account_lamports {
        return Err(ErrorCode::BidLowerThanStartingBid.into());
    }

    // check if the bid is higher than previous bid
    if listing_account.highest_bid_lamports.is_some()
        && listing_account.highest_bid_lamports.unwrap() >= bid_account_lamports
    {
        return Err(ErrorCode::BidLowerThanHighestBider.into());
    }

    // fetch token account of the bider
    let bidder_token = associated_token::get_associated_token_address(
        &ctx.accounts.bidder.key(),
        &listing_account.mint.clone(),
    );

    if bidder_token.key() != ctx.accounts.bidder_token.key() {
        return Err(ErrorCode::InvalidTokenAccount.into());
    }

    // transfer the fund
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.bidder.to_account_info(),
                to: ctx.accounts.bid_account_vault.to_account_info(),
            },
        ),
        bid_price_lamports,
    )?;

    bid_account.bidder_token = bidder_token.key();
    bid_account.bid_price_lamports = Some(bid_account_lamports);
    bid_account.fund_deposit = Some(true);

    listing_account.highest_bid_pda = Some(ctx.accounts.bid_account.key().clone());
    listing_account.highest_bid_lamports = Some(bid_account_lamports);
    listing_account.highest_bidder = Some(ctx.accounts.bidder.key());
    listing_account.highest_bidder_token = Some(bidder_token.key());

    Ok(())
}

#[derive(Accounts)]
pub struct BidEnglishAuction<'info> {
    #[account(mut)]
    pub nft_listing_account: Account<'info, NftListingData>,
    #[account(mut)]
    pub listing_account: Account<'info, EnglishAuctionListingData>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(mut)]
    pub bidder_token: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub seller_token: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub bid_account: Account<'info, EnglishAuctionListingBidData>,
    #[account(mut)]
    /// CHECK:
    pub bid_account_vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
}
