import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Staking } from "../target/types/staking";
import { Keypair, SystemProgram, PublicKey, SYSVAR_RENT_PUBKEY} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  setAuthority,
  AuthorityType,
  createAssociatedTokenAccount,
  getMinimumBalanceForRentExemptAccount,
  mintToChecked
} from "@solana/spl-token";
import { assert } from "chai";


const PREFIX_STAKING_SHCP = "shcp_staking";
const PREFIX_CONFIG = "shapz_config";


describe("staking", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.Provider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.Staking as Program<Staking>;
  const connection = program.provider.connection;

  // We need to set ourself into the appropriate context
  // We consider the player already own some shCP tokens
  // We consider the player already own 1 NFT
  // We consider the Shapz vault is already created
  //    - Have a token account to store shCP

  let shcp_mint_account_key: PublicKey;
  let nft_mint_account_key: PublicKey;
  let shcp_player_ata_key: PublicKey;
  let nft_ata_key: PublicKey;
  let schp_vault_ata_key: PublicKey;
  // the "nft_vault_ata" will be automatically created by the program
  const AUTHORITY = new anchor.web3.PublicKey("QeYrNiEd1NmBSzWJ28gCUEWKfpf8QU1nFz7gfHzgLP2");

  let player = Keypair.generate();  // The player (will also be the payer)
  let shapz_master = Keypair.generate();  // The shapz master (payer for setup MintAccount)
  // let mint_authority = Keypair.generate();  // The mint authority (own by us)

  it("Provide SOL to the player", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(player.publicKey, 10e9),
      "confirmed"
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(shapz_master.publicKey, 10e9),
      "confirmed"
    );

    console.log('----------------------------------------')
  });

  it("Setup MintAccount for shCP and 1 NFT", async () => {
    console.log("Create the MintAccount for shCP")
    shcp_mint_account_key = await createMint(
      connection,  // connection
      shapz_master,  // fee payer
      shapz_master.publicKey,  // mint authority
      shapz_master.publicKey,  // freeze authority
      9  // decimal
    );
    console.log(`mint: ${shcp_mint_account_key.toBase58()}`);

    console.log("Create the MintAccount for NFT")
    nft_mint_account_key = await createMint(
      connection,  // connection
      shapz_master,  // fee payer
      shapz_master.publicKey,  // mint authority
      shapz_master.publicKey,  // freeze authority
      0  // decimal
    );
    console.log(`mint: ${nft_mint_account_key.toBase58()}`);
    console.log('----------------------------------------')
  });

  it("Create the player shcp ata and NFT ata", async () => {
    console.log("Create the player shCP associated token account")
    shcp_player_ata_key = await createAssociatedTokenAccount(
      connection,  // connection
      player,  // fee payer
      shcp_mint_account_key,  // MintAccount
      player.publicKey,  // owner
    );
    
    console.log("Create the player NFT associated token account")
    nft_ata_key = await createAssociatedTokenAccount(
      connection,  // connection
      player,  // fee payer
      nft_mint_account_key,  // MintAccount
      player.publicKey,  // owner
    );

    console.log('----------------------------------------')
  });

  it("Give the player its NFT and some shCP tokens", async () => {
    console.log("Give the player its NFT")
    let txhash = await mintToChecked(
      connection,  // connection
      shapz_master,  // fee payer
      nft_mint_account_key,  // MintAccount
      nft_ata_key,  // AssociatedTokenAccount
      shapz_master.publicKey,  // MintAuthority
      1,  // amount
      0
    );
    console.log(`txhash: ${txhash}`);

    console.log('----------------------------------------')
  });

  it("Initialize the game vault", async () => {
    console.log('Calculate pda for autority over the vault')
    const [_shcp_vault_authority, _shcp_vault_authority_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("shcp_authority")),
      ],
      program.programId
    );

    console.log("Initialize the game vault")
    schp_vault_ata_key = await createAssociatedTokenAccount(
      connection,  // connection
      shapz_master,  // fee payer
      shcp_mint_account_key,  // MintAccount
      shapz_master.publicKey,  // owner
    );

    console.log("Give the vault some shCP tokens")
    let txhash = await mintToChecked(
      connection,  // connection
      shapz_master,  // fee payer
      shcp_mint_account_key,  // MintAccount
      schp_vault_ata_key,  // AssociatedTokenAccount
      shapz_master.publicKey,
      10000e9,  // amount
      9
    );

    // Create the pubkey for the config account
    console.log("Calculate PDA for the config account")
    const [_config_pda, _bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX_CONFIG)),
        AUTHORITY.toBuffer(),
      ],
      program.programId
    );

    console.log("Give the authority over the vault to the program")
    const tx = await program.rpc.globalInit({
      accounts: {
        shapzMaster: shapz_master.publicKey,
        shcpVaultAta: schp_vault_ata_key,
        authority: AUTHORITY,
        configAccount: _config_pda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [shapz_master],
    });

    console.log('----------------------------------------')
  })

  it("Staking a Compute Shapz", async () => {
    // The NFT vault WILL BE INITIATED by the program
    // But we need to create the public key using the same seed
    // as the program
    // The same goes for the stacking account

    // Create the pubkey for the player Stacking account
    console.log("Calculate PDA for the player staking account")
    const [_player_stacking_account, _psa_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX_STAKING_SHCP)),
        AUTHORITY.toBuffer(),
        player.publicKey.toBuffer(),
        nft_mint_account_key.toBuffer(),
      ],
      program.programId
    );
    
    const tx = await program.rpc.stakeShcp(
      {
        accounts: {
          player: player.publicKey,
          nftAtaAccount: nft_ata_key,
          nftMint: nft_mint_account_key,
          playerShcpClaimAccount: shcp_player_ata_key,
          shapzShcpVault: schp_vault_ata_key,
          authority: AUTHORITY,
          stackingAccount: _player_stacking_account,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [player]
      },
    );
    console.log('----------------------------------------')
  });

  it("Claiming the reward", async () => {
    // The config account hold the authority over the reward vault
    console.log("Calculate PDA for the config account")
    const [_config_pda, _bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX_CONFIG)),
        AUTHORITY.toBuffer(),
      ],
      program.programId
    );

    // We also need to access the player StakingAccount since some of 
    // its field will be updated
    console.log("Calculate PDA for the player staking account")
    const [_player_stacking_account, _psa_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX_STAKING_SHCP)),
        AUTHORITY.toBuffer(),
        player.publicKey.toBuffer(),
        nft_mint_account_key.toBuffer(),
      ],
      program.programId
    );

    // Wait for one seconds
    await new Promise(resolve => setTimeout(resolve, 1000));

    // There is no need for signer since the transfer is done from
    // the shapz shcp vault account, and the program already have
    // authority over it
    const tx = await program.rpc.claimShcpReward({
      accounts: {
        player: player.publicKey,
        playerShcpAta: shcp_player_ata_key,
        shcpVaultAta: schp_vault_ata_key,
        authority: AUTHORITY,
        nftMint: nft_mint_account_key,
        stackingAccount: _player_stacking_account,
        configAccount: _config_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: []
    });

    let schp_amount_per_seconds = 2314815;
    let schp_vault_init_amount = 10000e9;
    let max_expected_amount = schp_vault_init_amount - schp_amount_per_seconds

    let vault_token_amount = await connection.getTokenAccountBalance(schp_vault_ata_key);
    console.log(`vault_token_amount: ${vault_token_amount.value.amount}`);
    assert.isAtMost(vault_token_amount.value.uiAmount, max_expected_amount);

    console.log('----------------------------------------')
  });

  it("Unstaking", async () => {
    console.log("Calculate PDA for the player staking account")
    const [_player_stacking_account, _psa_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX_STAKING_SHCP)),
        AUTHORITY.toBuffer(),
        player.publicKey.toBuffer(),
        nft_mint_account_key.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.rpc.unstakeShcp({
      accounts : {
        player: player.publicKey,
        nftAtaAccount: nft_ata_key,
        nftMint: nft_mint_account_key,
        authority: AUTHORITY,
        stackingAccount: _player_stacking_account,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: []
    });

    console.log('----------------------------------------')
  
  });
});
