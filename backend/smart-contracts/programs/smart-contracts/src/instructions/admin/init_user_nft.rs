use {
    crate::constants::ADMIN, anchor_lang::prelude::*, anchor_spl::{
        metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata},
        token::{Mint, Token},
    }, mpl_token_metadata::{pda::find_metadata_account, state::DataV2}
};

#[derive(Accounts)]
pub struct InitUserNft<'info> {
    #[account(mut, address = ADMIN)]
    pub payer: Signer<'info>,

    // Create mint account
    // Same PDA as address of the account and mint/freeze authority
    #[account(
        init,
        seeds = [b"user-mint"],
        bump,
        payer = payer,
        mint::decimals = 0,
        mint::authority = user_nft_mint.key(),
        mint::freeze_authority = user_nft_mint.key(),

    )]
    pub user_nft_mint: Account<'info, Mint>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
        address=find_metadata_account(&user_nft_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_user_nft_handler(
    ctx: Context<InitUserNft>,
    token_name: String,
    token_symbol: String,
    token_uri: String,
) -> Result<()> {
    msg!("Creating metadata account");

    // PDA signer seeds
    let signer_seeds: &[&[&[u8]]] = &[&[b"user-mint", &[*ctx.bumps.get("user_nft_mint").unwrap()]]];

    // Cross Program Invocation (CPI) signed by PDA
    // Invoking the create_metadata_account_v3 instruction on the token metadata program
    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.user_nft_mint.to_account_info(),
                mint_authority: ctx.accounts.user_nft_mint.to_account_info(), // PDA is mint authority
                update_authority: ctx.accounts.user_nft_mint.to_account_info(), // PDA is update authority
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        DataV2 {
            name: token_name,
            symbol: token_symbol,
            uri: token_uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false, // Is mutable
        true,  // Update authority is signer
        None,  // Collection details
    )?;

    msg!("User NFT created successfully.");

    Ok(())
}