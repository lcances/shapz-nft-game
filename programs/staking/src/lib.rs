use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Token, Mint, TokenAccount, SetAuthority, set_authority,
        transfer, Transfer
        },
};
use spl_token::instruction::AuthorityType;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


const PREFIX_STAKING_SHCP: &[u8] = b"shcp_staking";
const PREFIX_CONFIG: &[u8] = b"shapz_config";


#[program]
pub mod staking {
    use super::*;

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
        
        // The PDA generated MUST be the same as the StakingAccount, since 
        // is will be the new authority of the NFT
        let (_nft_stake_pda, _nft_stake_pda_bump) = Pubkey::find_program_address(&[PREFIX_STAKING_SHCP,
            ctx.accounts.authority.key.as_ref(),
            ctx.accounts.player.key.as_ref(),
            ctx.accounts.nft_mint.to_account_info().key.as_ref(),
            ], ctx.program_id);

        set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),

                SetAuthority {
                    current_authority: ctx.accounts.player.to_account_info().clone(),
                    account_or_mint: ctx.accounts.nft_ata_account.to_account_info().clone(),
                }
            ),
            
            AuthorityType::AccountOwner,

            Some(_nft_stake_pda)
        )?;

        Ok(())
    }

    pub fn unstake_shcp(ctx: Context<UnstakeShcp>) -> Result<()> {
        // verify that the player (P) can claim the reward
        // We need to verify in the stacking account (SA) several point
        // P_pub == SA_P_pub
        if ctx.accounts.stacking_account.player_key != *ctx.accounts.player.key {
            return err!(ErrorCode::PlayerIsNotOwner);    
        }

        // P_mint_key == SA_mint_key  (Same mint pubkey for the NFT)
        if ctx.accounts.stacking_account.nft_mint_key != *ctx.accounts.nft_mint.to_account_info().key {
            return err!(ErrorCode::WrongNftKey);    
        }

        // The PDA generated MUST be the same as the StakingAccount, since 
        // it has (the staking account) the new authority of the NFT
        let (_stake_pda, _stake_pda_bump) = Pubkey::find_program_address(&[PREFIX_STAKING_SHCP,
            ctx.accounts.authority.key.as_ref(),
            ctx.accounts.player.key.as_ref(),
            ctx.accounts.nft_mint.to_account_info().key.as_ref(),
            ], ctx.program_id);

        let _stacking_account_seed = &[PREFIX_STAKING_SHCP,
            ctx.accounts.authority.key.as_ref(),
            ctx.accounts.player.key.as_ref(),
            ctx.accounts.nft_mint.to_account_info().key.as_ref(),
            &[_stake_pda_bump]];

        set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),

                SetAuthority {
                    current_authority: ctx.accounts.stacking_account.to_account_info().clone(),
                    account_or_mint: ctx.accounts.nft_ata_account.to_account_info().clone(),
                },

                &[&_stacking_account_seed[..]],
            ),
            
            AuthorityType::AccountOwner,

            Some(ctx.accounts.player.key())
        )?;

        Ok(())
    }


    pub fn claim_shcp_reward(ctx: Context<ClaimSchpReward>) -> Result<()> {

        // verify that the player (P) can claim the reward
        // We need to verify in the stacking account (SA) several point
        // P_pub == SA_P_pub
        if ctx.accounts.stacking_account.player_key != *ctx.accounts.player.key {
            return err!(ErrorCode::PlayerIsNotOwner);    
        }

        // P_mint_key == SA_mint_key  (Same mint pubkey for the NFT)
        if ctx.accounts.stacking_account.nft_mint_key != *ctx.accounts.nft_mint.to_account_info().key {
            return err!(ErrorCode::WrongNftKey);    
        }

        // Compute how many $shCP the player should receive
        let current_clock = Clock::get().unwrap().unix_timestamp;
        let clock_last_clain = ctx.accounts.stacking_account.claimed_at;

        let elapsed_seconds = current_clock - clock_last_clain;
        let shcp_lamport_amount_seconds = elapsed_seconds * ctx.accounts.stacking_account.shcp_amount_seconds;

        // Transfer the token
        // The config account hold authority to transfer the token
        // The account that will have authority over the vault is the config account
        // It's pubkey can be found using the seed bellow
        let (_config_pda, _bump) = Pubkey::find_program_address(&[PREFIX_CONFIG,
                ctx.accounts.authority.key.as_ref(),
            ], ctx.program_id);

        let _config_seed = &[PREFIX_CONFIG,
            ctx.accounts.authority.key.as_ref(),
            &[_bump]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),

                Transfer {
                    from: ctx.accounts.shcp_vault_ata.to_account_info().clone(),
                    to: ctx.accounts.player_shcp_ata.to_account_info().clone(),
                    authority: ctx.accounts.config_account.to_account_info().clone(),
                },

                &[&_config_seed[..]],

            ),
            shcp_lamport_amount_seconds as u64,
        );

        // Update the claimed_at field
        ctx.accounts.stacking_account.claimed_at = current_clock;

        Ok(())
    }

    pub fn global_init(ctx: Context<GlobalInit>) -> Result<()> {
        // This instruction is called only once, at the beginning of the program
        // It's purpose is to give authority over the $shCP ATA (the vault) to the program
        
        // The account that will have authority over the vault is the config account
        // It's pubkey can be found using the seed bellow
        let (_config_pda, _bump) = Pubkey::find_program_address(&[PREFIX_CONFIG,
                ctx.accounts.authority.key.as_ref(),
            ], ctx.program_id);
        
        // Then we need to declare a Cross Program Invocation (CPI)
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),

            anchor_spl::token::SetAuthority {
                current_authority: ctx.accounts.shapz_master.to_account_info().clone(),
                account_or_mint: ctx.accounts.shcp_vault_ata.to_account_info().clone(),
            }
        );

        // We can now transfer the authority to the program
        anchor_spl::token::set_authority(
            cpi_ctx,
            AuthorityType::AccountOwner,
            Some(_config_pda),
        )?;

        // Wew can now mark the vault as initialized
        ctx.accounts.config_account.shcp_vault_is_initialized = 1;
        ctx.accounts.config_account.authority = *ctx.accounts.authority.key;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct GlobalInit<'info> {
    #[account(mut)]
    pub shapz_master: Signer<'info>,

    // The $shCP reward vault that the program will take authority over
    #[account(mut)]
    pub shcp_vault_ata: Account<'info, TokenAccount>,

    ///CHECK:
    pub authority: AccountInfo<'info>,

    #[account(
        init,
        payer = shapz_master,
        space = 8 + ConfigAccount::LEN,
        seeds = [
            PREFIX_CONFIG,  // PREFIX_SHCP_STACKING
            authority.key().as_ref(),
        ],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct StakeShcp<'info> {

    // The player, obviously
    #[account(mut)]
    pub player: Signer<'info>,
    
    // The player's nft ata, it should already exist since the player own the NFT.
    // Then the mint pubkey of the NFT 
    #[account(mut)]
    pub nft_ata_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,

    // The player's $shCP claim account, it should already exist since the player
    // already own $shCP. That where the $shCP will be claim in
    #[account(mut)]
    pub player_shcp_claim_account: Account<'info, TokenAccount>,

    // Then the vault $shCP that also already exist. It is where all the token are
    // stored.
    #[account(mut)]
    pub shapz_shcp_vault: Account<'info, TokenAccount>,

    // Staking mean that we are going to give to the program
    // authority over the NFT, we need then the authority key
    /// CHECK:
    pub authority: AccountInfo<'info>,
    
    // The stacking account need to be created and will store all the information
    // needed to link the player with the staked NFT.
    // The address is calculated using a PDA created from a
    //    - a prefix
    //    - The authority pubkey
    //    - the player's pubkey
    //    - the nft mint pubkey
    #[account(
        init,
        payer = player,
        space = 8 + StakingAccount::LEN,
        seeds = [
            PREFIX_STAKING_SHCP,
            authority.key().as_ref(),
            player.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub stacking_account: Box<Account<'info, StakingAccount>>,

    // Since we are creating a data account, we need to provide
    // some SOL to pay the rent
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

    /// CHECK:
    pub authority: AccountInfo<'info>,

    pub nft_mint: Account<'info, Mint>,
    #[account(
        seeds = [
            PREFIX_STAKING_SHCP,
            authority.key().as_ref(),
            player.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub stacking_account: Box<Account<'info, StakingAccount>>,

    // The config account should the one that have authority over the vault
    #[account(
        seeds = [
            PREFIX_CONFIG,  // PREFIX_SHCP_STACKING
            authority.key().as_ref(),
        ],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnstakeShcp<'info> {
    /// The player, but do not need to sign the transaction
    /// CHECK:
    #[account(mut)]
    pub player: AccountInfo<'info>,

    // The player's nft ata, it should already exist since the player own the NFT.
    // Then the mint pubkey of the NFT 
    #[account(mut)]
    pub nft_ata_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,

    // UN-Staking mean that we are going to give to the PLAYER
    // authority over the NFT, we need then the authority key
    /// CHECK:
    pub authority: AccountInfo<'info>,

    // The stacking account linked to the player. it is necessary
    // to run some verification and need to be closed.
    // The address is calculated using a PDA created from a
    //    - a prefix
    //    - The authority pubkey
    //    - the player's pubkey
    //    - the nft mint pubkey
    #[account(
        seeds = [
            PREFIX_STAKING_SHCP,
            authority.key().as_ref(),
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

#[account]
pub struct ConfigAccount {
    pub shcp_vault_is_initialized: u8,
    pub shfec_vault_is_initialized: u8,
    pub authority: Pubkey,
}

impl ConfigAccount {
    pub const LEN: usize = 1 +  // shcp_vault_is_initialized
        1 +  // shfec_vault_is_initialized
        32;  // authority
}


#[error_code]
pub enum ErrorCode {
    #[msg("The player is not the owner of the StakingAccount")]
    PlayerIsNotOwner,
    #[msg("Wrong NFT public key")]
    WrongNftKey,
}