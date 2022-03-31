use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, Mint, TokenAccount, SetAuthority, set_authority,
        transfer, Transfer
        },
};
use spl_token::instruction::AuthorityType;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod staking {
    use super::*;

    const SCHP_STACKINGACCOUNT_SEED: &[u8] = b"shcp_stacking";
    const SCHP_AUTHORITY_SEED: &[u8] = b"shcp_authority";

    pub fn stake_shcp(ctx: Context<StakeShcp>) -> Result<()> {
        let clock = Clock::get().unwrap().unix_timestamp;

        // Gain is 200 shCP per day
        // We want to compute the amount of shCP generated every seconds
        // There is 3600 * 24 = 86400 seconds in a day
        // The $shCP token have 9 decimals, and we will transfer "lamport"
        // so we need to multiply the final amount by 1^9
        // 200 / 86400 = 0.002314815
        let shcp_lamport_amount_seconds = 002314815;  // 200 shCP/j = 200 ||| (24 * 3600 seconds) * 1e9  (9 digit after coma)

        
        // The Staking account should have been created directly
        // thanks to the macro `init`
        ctx.accounts.stacking_account.player_key = *ctx.accounts.player.key;
        ctx.accounts.stacking_account.nft_ata_key = *ctx.accounts.nft_ata_account.to_account_info().key;
        ctx.accounts.stacking_account.player_shcp_claim_account_key = *ctx.accounts.player_shcp_claim_account.to_account_info().key;
        ctx.accounts.stacking_account.nft_mint_key = *ctx.accounts.nft_mint.to_account_info().key;
        ctx.accounts.stacking_account.shcp_amount_seconds = shcp_lamport_amount_seconds;
        ctx.accounts.stacking_account.created_at = clock;
        ctx.accounts.stacking_account.claimed_at = clock;

        // Now that we have filled every field of the StakingAccount, we need
        // the program to have authority to manipulate the NFT.
        let (_authority, _authority_bump) = Pubkey::find_program_address(&[SCHP_STACKINGACCOUNT_SEED], ctx.program_id);

        set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),

                SetAuthority {
                    current_authority: ctx.accounts.player.to_account_info().clone(),
                    account_or_mint: ctx.accounts.nft_ata_account.to_account_info().clone(),
                }
            ),
            
            AuthorityType::AccountOwner,

            Some(_authority)
        )?;

        Ok(())
    }

    pub fn claim_shcp_reward(ctx: Context<ClaimSchpReward>) -> Result<()> {
        // Compute how many $shCP the player should receive
        let current_clock = Clock::get().unwrap().unix_timestamp;
        let clock_last_clain = ctx.accounts.stacking_account.claimed_at;

        let elapsed_seconds = current_clock - clock_last_clain;
        let shcp_lamport_amount_seconds = ctx.accounts.stacking_account.shcp_amount_seconds;

        // Transfer
        let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(&[SCHP_AUTHORITY_SEED], ctx.program_id);
        let authority_seeds = &[&SCHP_AUTHORITY_SEED[..], &[vault_authority_bump]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),

                Transfer {
                    from: ctx.accounts.shcp_vault_ata.to_account_info().clone(),
                    to: ctx.accounts.player_shcp_ata.to_account_info().clone(),
                    authority: ctx.accounts.authority.to_account_info().clone(),
                },

                &[&authority_seeds[..]],

            ),
            shcp_lamport_amount_seconds as u64,
        );

        Ok(())
    }

    pub fn global_init(ctx: Context<GlobalInit>) -> Result<()> {
        let (_authority, _authority_bump) = Pubkey::find_program_address(&[SCHP_AUTHORITY_SEED], ctx.program_id);
        // let authority_seeds = &[SCHP_AUTHORITY_SEED[..], &[vault_authority_bump]];

        set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),

                SetAuthority {
                    current_authority: ctx.accounts.shapz_master.to_account_info().clone(),
                    account_or_mint: ctx.accounts.shcp_vault_ata.to_account_info().clone(),
                }
            ),
            
            AuthorityType::AccountOwner,

            Some(_authority)
        )?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct GlobalInit<'info> {
    #[account(mut)]
    pub shapz_master: Signer<'info>,
    #[account(mut)]
    pub shcp_vault_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct StakeShcp<'info> {

    // The player, obviously
    #[account(mut)]
    pub player: Signer<'info>,
    //
    // The player's nft ata, it should already exist since the player own the NFT.
    // Then the mint pubkey of the NFT 
    #[account(mut)]
    pub nft_ata_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    //
    // The player's $shCP claim account, it should already exist since the player
    // already own $shCP. That where the account will be claim in
    // Then the vault $shCP that also already exist. It where all the token are
    // stored.
    #[account(mut)]
    pub player_shcp_claim_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shapz_shcp_vault: Account<'info, TokenAccount>,
    //
    // The stacking account need to be created and will store all the information
    // to link the player with the staked NFT
    #[account(
        init,
        payer = player,
        space = 8 + StakingAccount::LEN,
        seeds = [
            b"shcp_stacking",
            player.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub stacking_account: Box<Account<'info, StakingAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimSchpReward<'info> {
    /// CHECK: 
    #[account(mut)]
    pub player: AccountInfo<'info>,
    #[account(mut)]
    pub player_shcp_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shcp_vault_ata: Account<'info, TokenAccount>,
    /// CHECK: This is the authority account, need to add checks
    #[account(
        seeds = [b"shcp_authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        seeds = [
            b"shcp_stacking",
            player.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub stacking_account: Box<Account<'info, StakingAccount>>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct StakingAccount {
    pub player_key: Pubkey,
    pub nft_ata_key: Pubkey,
    pub player_shcp_claim_account_key: Pubkey,
    pub nft_mint_key: Pubkey,
    pub shcp_amount_seconds: i64,
    pub created_at: i64,
    pub claimed_at: i64,
}

impl StakingAccount {
    pub const LEN: usize = 32 +  // player key
        32 +  // player nft account key
        32 +  // player shcp account key
        32 +  // shapz nft account key
        32 +  // nft mint key
        8 +  // shcp amount seconds
        8 +  // created at
        8;  // claimed at
}


#[error_code]
pub enum ErrorCode {
    #[msg("The player is not the owner of the StakingAccount")]
    PlayerIsNotOwner,
    #[msg("Wrong NFT public key")]
    WrongNftKey,
}