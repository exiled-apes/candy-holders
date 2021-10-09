// use std::{sync::mpsc::channel, thread};
use anchor_lang::AccountDeserialize;
use gumdrop::Options;
use nft_candy_machine::{CandyMachine, Config};
use rusqlite::{params, Connection};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::account::ReadableAccount;
use solana_transaction_status::UiTransactionEncoding;
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

// #[derive(Clone, Debug, Options)]
// struct FindMetadataAccounts {
//     #[options(help = "update authority address")]
//     update_authority: String,
// }

#[derive(Clone, Debug, Options)]
struct ListTokenParams {
    #[options(help = "update authority address")]
    update_authority: String,
}

#[derive(Clone, Debug, Options)]
struct MineTokensByUpdateAuthority {
    #[options(help = "update authority address")]
    update_authority: String,
}

#[derive(Clone, Debug, Options)]
struct ShowCandyMachine {
    #[options(free)]
    args: Vec<String>,
}

#[derive(Clone, Debug, Options)]
enum Command {
    // FindMetadataAccounts(FindMetadataAccounts),
    ShowCandyMachine(ShowCandyMachine),
    ListTokens(ListTokenParams), // eventually remove / retire
    // MineTokensByCandyMachine(MineTokensWithUpdateAuthority),
    MineTokensByUpdateAuthority(MineTokensByUpdateAuthority),
    // ListHolders(),
}

fn main() {
    let app_options = AppOptions::parse_args_default_or_exit();
    if app_options.help {
        println!("Usage: [OPTIONS] [COMMAND] [ARGUMENTS]");
        println!();
        println!("{}", AppOptions::usage());
        println!();

        println!("Available commands:");
        println!();
        println!("{}", Command::usage());
        return;
    }

    match app_options.clone().command {
        Some(command) => match command {
            Command::ShowCandyMachine(opts) => show_candy_machine(app_options, opts),
            // Command::FindMetadataAccounts(opts) => find_metadata_accounts(app_options, opts),
            Command::ListTokens(opts) => list_tokens(app_options, opts),
            Command::MineTokensByUpdateAuthority(opts) => {
                mine_tokens_by_update_authority(app_options, opts)
            }
            // Command::ListTransactions(_) => todo!(), // list_transactions(app_options, opts),
            // Command::ListHolders => todo!(),
        },
        None => todo!(),
    }
}

fn list_tokens(app_options: AppOptions, opts: ListTokenParams) {
    let client = RpcClient::new(app_options.rpc_url);

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

    let program_accounts = client
        .get_program_accounts_with_config(pubkey, cfg)
        .expect("could not get program accounts");

    for (pubkey, _account) in program_accounts {
        let sigs = client.get_signatures_for_address(&pubkey);
        if let Err(err) = sigs {
            eprintln!("could not get signatures {} {:?}", pubkey, err);
            continue;
        }

        let sigs = sigs.unwrap();
        if sigs.len() >= 1000 {
            eprintln!("too many sigs {} {}", pubkey, sigs.len());
            continue;
        }
        if sigs.len() < 1 {
            eprintln!("not enough sigs {} {}", pubkey, sigs.len());
            continue;
        }

        let sig = sigs.last().unwrap();
        let sig = sig.signature.parse().unwrap();

        let tx = client.get_transaction(&sig, UiTransactionEncoding::Base58);
        if let Err(err) = tx {
            eprintln!("couldn't get transaction {} {}", sig, err);
            continue;
        }

        let tx = tx.unwrap().transaction;
        let tx = tx.transaction.decode();
        if let None = tx {
            eprintln!("could not decode sig tx {} {}", pubkey, sig);
            continue;
        }

        let tx = tx.unwrap();
        let msg = tx.message();
        if msg.instructions.len() != 5 {
            eprintln!(
                "invalid instruction count {} {}",
                pubkey,
                msg.instructions.len()
            );
            continue;
        }

        let token_address = msg.account_keys.get(1);
        if let None = token_address {
            eprintln!("couldn't get token address {}", sig);
            continue;
        }

        let token_address = token_address.unwrap();
        println!("{}", token_address);
    }
}

fn mine_tokens_by_update_authority(app_options: AppOptions, opts: MineTokensByUpdateAuthority) {
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
        )
        .expect("could not insert token!!");
    }
}

fn show_candy_machine(app_options: AppOptions, opts: ShowCandyMachine) {
    let client = RpcClient::new(app_options.rpc_url);

    for arg in opts.args {
        let cm_id = &arg.parse().expect("could not parse candy machine pubkey");

        let cm_account = client
            .get_account(cm_id)
            .expect("could not fetch candy machine account");

        let mut cm_data = cm_account.data();
        let cm = CandyMachine::try_deserialize(&mut cm_data)
            .expect("could not deserialize candy machine data");

        let cfg_acct = client
            .get_account(&cm.config)
            .expect("could not fetch config account");

        let mut cfg_data = cfg_acct.data();
        let config =
            Config::try_deserialize(&mut cfg_data).expect("could not deserialize config data");

        eprintln!("candy_machine          {}", cm_id);
        eprintln!("  authority            {}", cm.authority);
        eprintln!("  wallet               {}", cm.wallet);
        eprintln!("  token_mint           {:?}", cm.token_mint);
        eprintln!("  items_redeemed       {}", cm.items_redeemed);

        // data
        eprintln!("  data.uuid            {}", cm.data.uuid);
        eprintln!("  data.price           {}", cm.data.price);
        eprintln!("  data.items_available {}", cm.data.items_available);
        eprintln!("  data.go_live_date    {:?}", cm.data.go_live_date);

        // config
        eprintln!("  config.authority     {}", config.authority);
        eprintln!("  config.data.uuid     {}", config.data.uuid);
        eprintln!("  config.data.symbol   {}", config.data.symbol);
        eprintln!(
            "  config.data.seller_fee_basis_points {}",
            config.data.seller_fee_basis_points
        );
        for creator in config.data.creators {
            eprintln!(
                "  config.data.creators {}, {}",
                creator.address, creator.share
            );
        }
        eprintln!("  config.data.max_supply {}", config.data.max_supply);
        eprintln!("  config.data.is_mutable {}", config.data.is_mutable);
        eprintln!(
            "  config.data.retain_authority {}",
            config.data.retain_authority
        );
        eprintln!(
            "  config.data.max_number_of_lines {}",
            config.data.max_number_of_lines
        );
    }
}

fn default_rpc_url() -> String {
    "https://api.mainnet-beta.solana.com".to_owned()
}

fn default_db_path() -> String {
    "candy-holders.db".to_owned()
}

// #[derive(Clone, Debug, Options)]
// struct ListTransactionsOpts {
//     #[options(free)]
//     args: Vec<String>,
//     #[options(help = "search for transactions before this one")]
//     before: Option<String>,
// }

// fn find_metadata_accounts(app_options: AppOptions, opts: FindMetadataAccounts) {
//     let client = RpcClient::new(app_options.rpc_url);

//     let cfg = RpcProgramAccountsConfig {
//         account_config: RpcAccountInfoConfig {
//             encoding: Some(UiAccountEncoding::Base64Zstd),
//             ..RpcAccountInfoConfig::default()
//         },
//         filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
//             offset: 1,
//             bytes: MemcmpEncodedBytes::Binary(opts.update_authority),
//             encoding: None,
//         })]),
//         ..RpcProgramAccountsConfig::default()
//     };

//     let pubkey = &"metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//         .parse()
//         .unwrap();

//     let program_accounts = client
//         .get_program_accounts_with_config(pubkey, cfg)
//         .expect("could not get program accounts");

//     for (pubkey, _account) in program_accounts {
//         println!("{}", pubkey);

//         // Getting Metadata:
//         // let mut buf = account.data();
//         // let metadata = Metadata::deserialize(&mut buf).expect("could not deserialize metadata");
//         // println!("\t{}", metadata.update_authority);
//     }
// }
