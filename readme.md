# Stacking

## List of program accounts

### The rewards vault <TokenAccount>
The reward vaults hold the tokens that will be distributed to the players.
They are created and managed by the program.
Their public key will be generated using a seed.
This seed is computed using a prefix, the authority public key
```
 ┌────────────────────┐   ┌─────────────────────┐
 │                    │   │                     │
 │  $shCP Vault       │   │  $shFEC Vault       │
 │   (VAULT_PREFIX    │   │   (VAULT_PREFIX     │
 │    MintAccount.key │   │    MintAccount.key) │
 │                    │   │                     │
 └────────────────────┘   └─────────────────────┘
```

### The stacking account
When a player stake his NFT, a stacking account is created. This account hold all the informations required to identify the player, the NFT and also the token account for the reward vault and the player token ATA.
```
 ┌──────────────────────────┐
 │                          │
 │ StakingAccount           │
 │   - Player public key    │
 │   - NFT mint key         │
 │   - Token reward vault   │
 │   - Player token account │
 │   - Creation date        │
 │   - Last claim date      │
 │                          │
 └──────────────────────────┘
```

### Configuration account
The configuration account will hold some state about the program.
For exemple, when a reward vault is created, a specific field will
be marked as True, and so on.
```
 ┌───────────────────────────────────┐
 │                                   │
 │ ConfigAccount                     │
 │   - shcp_vault_initialized  (0|1) │
 │   - shFEC_vault_initialized (0|1) │
 │   - authority plublic key         │
 │                                   │
 └───────────────────────────────────┘
 ```


# Function 
## global init
`pub fn global_init(ctx: Context<GlobalInit>) -> Result<()> {...}`

The global init function goal is to initialize the game.
To do so, the config account is created and will hold the basic information
define above.

The config account public key is created using a seed form from: A prefix, and the authority public key. This account is the one that **will have authority over all
program accounts**

