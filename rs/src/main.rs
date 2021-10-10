use gumdrop::Options;
use rusqlite::{params, Connection, Result};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{rpc_client::RpcClient, rpc_config::{RpcAccountInfoConfig, RpcLargestAccountsConfig, RpcLargestAccountsFilter, RpcProgramAccountsConfig}, rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType}};
use solana_transaction_status::UiTransactionEncoding;

// use solana_sdk::account::ReadableAccount;
// use nft_candy_machine::{CandyMachine, Config};
// use spl_token_metadata::state::Metadata;
// use solana_sdk::commitment_config::CommitmentConfig;
// use solana_sdk::signature::Signature;
// use solana_transaction_status::UiTransactionEncoding;

#[derive(Clone, Debug, Options)]
struct AppOptions {
    #[options(help = "print help")]
    help: bool,

    #[options(help = "Solana rpc server url", default_expr = "default_rpc_url()")]
    rpc_url: String,

    #[options(help = "slite db path", default_expr = "default_db_path()")]
    db_path: String,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Options)]
struct ByCandyMachineId {
    #[options(help = "candy machine id")]
    candy_machine_id: String,
}

#[derive(Clone, Debug, Options)]
struct ByUpdateAuthority {
    #[options(help = "update authority address")]
    update_authority: String,
}

#[derive(Clone, Debug, Options)]
enum Command {
    MineTokensByUpdateAuthority(ByUpdateAuthority),
    MineHoldersByUpdateAuthority(ByUpdateAuthority),
}

fn main() -> Result<()> {
    let app_options = AppOptions::parse_args_default_or_exit();
    if app_options.help {
        println!("Usage: [OPTIONS] [COMMAND] [ARGUMENTS]");
        println!();
        println!("{}", AppOptions::usage());
        println!();

        println!("Available commands:");
        println!();
        println!("{}", Command::usage());
        return Ok(());
    }

    match app_options.clone().command {
        Some(command) => match command {
            Command::MineHoldersByUpdateAuthority(opts) => {
                mine_holders_by_update_authority(app_options, opts)
            }
            Command::MineTokensByUpdateAuthority(opts) => {
                mine_tokens_by_update_authority(app_options, opts)
            }
        },
        None => todo!(),
    }
}

#[derive(Debug)]
struct TokenRow {
    token_address: String,
}

fn mine_holders_by_update_authority(
    app_options: AppOptions,
    opts: ByUpdateAuthority,
) -> Result<()> {
    mine_tokens_by_update_authority(app_options.clone(), opts)?;

    let client = RpcClient::new(app_options.rpc_url);
    let db = Connection::open(app_options.db_path)?;

    db.execute(
        "create table if not exists holders (
             token_address text primary key,
             holders_address text unique
         )",
        params![],
    )
    .expect("could not create holders table");

    let mut stmt = db.prepare("SELECT token_address FROM tokens;")?;

    let token_row_iter = stmt.query_map([], |row| {
        Ok(TokenRow {
            token_address: row.get(0)?,
        })
    })?;

    // loop over tokens
    for token_row in token_row_iter {
        let token_row = token_row.unwrap();
        let token_address = token_row.token_address;

        // skip it if it's already in the holders table (relies on externally managing the state of this table)
        let count: Result<u8, rusqlite::Error> = db.query_row(
            "select count(*) from tokens where token_address = ?1",
            params![token_address.clone()],
            |row| row.get(0),
        );
        let count = count.unwrap();
        if count >= 1u8 {
            continue;
        }

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // WHAT THE FVCK ??? THERE IS NO getTokenLargestAccounts ???
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        // client.get_token_account_balance_with_commitment(pubkey, commitment_config)

        // let config = RpcLargestAccountsConfig {
        //     commitment: None,
        //     filter: Some(RpcLargestAccountsFilter::Circulating),
        // };

        // let largest_accounts =client.get_largest_accounts_with_config(config)?;

        // find the wallet with the largest balance and save it
        // eprintln!("{}", token_address);
    }

    Ok(())
}

fn mine_tokens_by_update_authority(app_options: AppOptions, opts: ByUpdateAuthority) -> Result<()> {
    let client = RpcClient::new(app_options.rpc_url);
    let db = Connection::open(app_options.db_path).expect("could not open db");

    db.execute(
        "create table if not exists tokens (
             token_address text primary key,
             metadata_address text unique,
             genesis_signature text unique
         )",
        params![],
    )
    .expect("could not create tokens table");

    let cfg = RpcProgramAccountsConfig {
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64Zstd),
            ..RpcAccountInfoConfig::default()
        },
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: 1,
            bytes: MemcmpEncodedBytes::Binary(opts.update_authority),
            encoding: None,
        })]),
        ..RpcProgramAccountsConfig::default()
    };

    let pubkey = &"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        .parse()
        .unwrap();

    let metadata_accounts = client
        .get_program_accounts_with_config(pubkey, cfg)
        .expect("could not get program accounts");

    for (metadata_address, _account) in metadata_accounts {
        eprint!("{}", ".");

        let count: Result<u8, rusqlite::Error> = db.query_row(
            "select count(*) from tokens where metadata_address = ?1",
            params![metadata_address.to_string()],
            |row| row.get(0),
        );

        let count = count.unwrap();
        if count >= 1u8 {
            continue;
        }

        let sigs = client.get_signatures_for_address(&metadata_address);
        if let Err(err) = sigs {
            eprintln!("\ncould not get signatures {} {:?}", pubkey, err);
            continue;
        }

        let sigs = sigs.unwrap();
        if sigs.len() >= 1000 {
            eprintln!("\ntoo many sigs {} {}", pubkey, sigs.len());
            continue;
        }
        if sigs.len() < 1 {
            eprintln!("\nnot enough sigs {} {}", pubkey, sigs.len());
            continue;
        }

        let sig = sigs.last().unwrap();
        let sig = sig.signature.parse().unwrap();

        let tx = client.get_transaction(&sig, UiTransactionEncoding::Base58);
        if let Err(err) = tx {
            eprintln!("\ncouldn't get transaction {} {}", sig, err);
            continue;
        }

        let tx = tx.unwrap().transaction;
        let tx = tx.transaction.decode();
        if let None = tx {
            eprintln!("\ncould not decode sig tx {} {}", pubkey, sig);
            continue;
        }

        let tx = tx.unwrap();
        let msg = tx.message();
        if msg.instructions.len() != 5 {
            eprintln!(
                "\ninvalid instruction count {} {}",
                pubkey,
                msg.instructions.len()
            );
            continue;
        }

        let token_address = msg.account_keys.get(1);
        if let None = token_address {
            eprintln!("\ncouldn't get token address {}", sig);
            continue;
        }

        let token_address = token_address.unwrap();

        db.execute(
            "INSERT INTO tokens
            (token_address, metadata_address, genesis_signature) values
            (?1           , ?2              , ?3               )",
            params![
                token_address.to_string(),
                metadata_address.to_string(),
                sig.to_string(),
            ],
        )?;
    }

    Ok(())
}

fn default_rpc_url() -> String {
    "https://api.mainnet-beta.solana.com".to_owned()
}

fn default_db_path() -> String {
    "candy-holders.db".to_owned()
}
