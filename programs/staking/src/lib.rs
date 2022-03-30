use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // For the initialization of the staking
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

// #[derive(Accounts)]
// pub struct Initialize {
//     #[account(mut)]
//     pub player: Signer<'info>,
//     #[account(mut)]
//     pub player_deposit_nft_account: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub player_shcp_claim_account: Account<'info, TokenAccount>,
//     #[account(
//         init,
//         payer = player,
//         space = 8 + ShapzAccount::LEN
//     )]
//     pub stacking_account: Account<'info, StakingAccount>,
//     #[account(mut)]
//     pub shapz_shcp_vault: Account<'info, TokenAccount>,
//     #[account(
//         init_if_needed,
//         payer = player,
//     )]
//     pub shapz_nft_account: Account<'info, TokenAccount>,
// }

// #[account]
// pub struct StakingAccount {
//     pub player_key: Pubkey,
//     pub player_nft_account_key: Pubkey,
//     pub shapz_nft_account_key: Pubkey,
//     pub player_shcp_account_key: Pubkey,
//     pub nft_mint_key: Pubkey,
//     pub shcp_amount_seconds: u64,
//     pub created_at: i64,
//     pub claimed_at: i64,
// }

// impl StakingAccount {
//     pub const LEN: usize = 32 +  // player key
//         32 +  // player nft account key
//         32 +  // player shcp account key
//         32 +  // shapz nft account key
//         32 +  // nft mint key
//         8 +  // shcp amount seconds
//         8 +  // created at
//         8;  // claimed at
// }