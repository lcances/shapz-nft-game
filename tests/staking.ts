import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Staking } from "../target/types/staking";
import { Keypair, SystemProgram, PublicKey, SYSVAR_RENT_PUBKEY} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  getMinimumBalanceForRentExemptAccount,
  mintToChecked
} from "@solana/spl-token";

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
  let nft_player_ata_key: PublicKey;
  let schp_vault_ata_key: PublicKey;
  // the "nft_vault_ata" will be automatically created by the program

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
    nft_player_ata_key = await createAssociatedTokenAccount(
      connection,  // connection
      player,  // fee payer
      nft_mint_account_key,  // MintAccount
      player.publicKey,  // owner
    );
  });

  it("Give the player its NFT and some shCP tokens", async () => {
    console.log("Give the player its NFT")
    let txhash = await mintToChecked(
      connection,  // connection
      shapz_master,  // fee payer
      nft_mint_account_key,  // MintAccount
      nft_player_ata_key,  // AssociatedTokenAccount
      shapz_master.publicKey,  // MintAuthority
      1,  // amount
      0
    );
    console.log(`txhash: ${txhash}`);

    console.log("Give the player some shCP tokens")
    txhash = await mintToChecked(
      connection,  // connection
      shapz_master,  // fee payer
      shcp_mint_account_key,  // MintAccount
      shcp_player_ata_key,  // AssociatedTokenAccount
      shapz_master.publicKey,
      10e9,  // amount
      9
    );
  });

  it("Initialize the game vault", async () => {
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
  })

  it("Staking a Compute Shapz", async () => {
    // The NFT vault WILL BE INITIATED by the program
    // But we need to create the public key using the same seed
    // as the program
    // The same goes for the stacking account

    // Create the pubkey for the NFT vault from a seed
    const [_vault_nft_ata, _bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("nft_vault_ata")),
        player.publicKey.toBuffer(),
        nft_mint_account_key.toBuffer(),
      ],
      program.programId
    );
    console.log(`vault_nft_ata: ${_vault_nft_ata.toBase58()}`);

    // Create the pubkey for the player Stacking account
    const [_player_stacking_account, _psa_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("shcp_stacking")),
        player.publicKey.toBuffer(),
        nft_mint_account_key.toBuffer(),
      ],
      program.programId
    );
    console.log(`player_stacking_account: ${_player_stacking_account.toBase58()}`);

    console.log(`player address: ${player.publicKey.toBase58()}`);
    
    const tx = await program.rpc.stakeShcp(
      {
        accounts: {
          player: player.publicKey,
          playerNftAtaAccount: nft_player_ata_key,
          shapzNftAccount: _vault_nft_ata,
          nftMint: nft_mint_account_key,
          playerShcpClaimAccount: shcp_player_ata_key,
          shapzShcpVault: schp_vault_ata_key,
          stackingAccount: _player_stacking_account,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [player]
      },
    )

  });

});
