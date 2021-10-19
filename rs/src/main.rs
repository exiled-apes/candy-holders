use borsh::de::BorshDeserialize;
use gumdrop::Options;
use rusqlite::{params, Connection, Result};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    account::ReadableAccount,
    signer::{keypair::read_keypair_file, Signer},
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use spl_token_metadata::{
    instruction::update_metadata_accounts,
    state::{Data, Metadata},
};

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
struct ByCandyMachineIdArgs {
    #[options(help = "candy machine id")]
    candy_machine_id: String,
}

#[derive(Clone, Debug, Options)]
struct ByUpdateAuthorityArgs {
    #[options(help = "update authority address")]
    update_authority: String,
}

#[derive(Clone, Debug, Options)]
struct MineTokenMetadataArgs {}

#[derive(Clone, Debug, Options)]
struct RepairMetabaesArgs {
    #[options(help = "path to keypair file")]
    keypair_path: String,
}

#[derive(Clone, Debug, Options)]
struct ReplaceUpdateAuthorityArgs {
    #[options(help = "path to a keypair file")]
    current_update_authority: String,

    new_update_authority: String,
}

#[derive(Clone, Debug, Options)]
enum Command {
    MineTokensByUpdateAuthority(ByUpdateAuthorityArgs),
    MineTokenMetadata(MineTokenMetadataArgs),
    RepairMetabaes(RepairMetabaesArgs),
    ReplaceUpdateAuthority(ReplaceUpdateAuthorityArgs),
}

#[derive(Debug)]
struct TokenRow {
    token_address: String,
    metadata_address: String,
    genesis_signature: String,
    genesis_block_time: i64,
}

#[derive(Debug)]
struct MetadataRow {
    token_address: String,
    metadata_address: String,
    key: String,
    update_authority: String,
    mint: String,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    primary_sale_happened: bool,
    is_mutable: bool,
    edition_nonce: Option<u8>,
}

#[derive(Debug)]
struct RepairRow {
    token_address: String,
    metadata_address: String,
    old_name: String,
    new_name: String,
    old_uri: String,
    new_uri: String,
}

#[derive(Debug)]
struct CreatorRow {
    metadata_address: String,
    address: String,
    share: u8,
    idx: u8,
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
            Command::MineTokensByUpdateAuthority(opts) => {
                mine_tokens_by_update_authority(app_options, opts)
            }
            Command::MineTokenMetadata(opts) => mine_token_metadata(app_options, opts),
            Command::RepairMetabaes(opts) => repair_metabaes(app_options, opts),
            Command::ReplaceUpdateAuthority(opts) => replace_update_authority(app_options, opts),
        },
        None => todo!(),
    }
}

fn replace_update_authority(
    app_options: AppOptions,
    opts: ReplaceUpdateAuthorityArgs,
) -> Result<()> {
    let client = RpcClient::new(app_options.rpc_url);
    let db = Connection::open(app_options.db_path)?;

    let mut stmt = db.prepare(
        "SELECT token_address, metadata_address, old_name, new_name, old_uri, new_uri FROM repairs ORDER BY token_address",
    )?;

    let repair_row_iter = stmt.query_map([], |row| {
        Ok(RepairRow {
            token_address: row.get(0)?,
            metadata_address: row.get(1)?,
            old_name: row.get(2)?,
            new_name: row.get(3)?,
            old_uri: row.get(4)?,
            new_uri: row.get(5)?,
        })
    })?;

    for repair_row in repair_row_iter {
        let repair_row = repair_row.unwrap();

        let metadata_address = &repair_row
            .metadata_address
            .parse()
            .expect("could not parse metadata_address");

        let account = client
            .get_account(metadata_address)
            .expect("could not fetch metadata account");

        let mut buf = account.data();
        let metadata = Metadata::deserialize(&mut buf).expect("could not deserialize metadata");

        let current_update_authority = read_keypair_file(opts.current_update_authority.clone())
            .expect("could not read keypair file");

        let new_update_authority = opts.new_update_authority.parse().unwrap();

        // eprintln!("repair_row.token_address  {}", repair_row.token_address);
        // eprintln!("metadata.update_authority {}", metadata.update_authority);
        // eprintln!(
        //     "current_update_authority  {}",
        //     current_update_authority.pubkey()
        // );
        // eprintln!("new_update_authority      {}", new_update_authority);

        if {
            metadata.update_authority == current_update_authority.pubkey()
                && metadata.update_authority != new_update_authority
        } {
            eprintln!("repair_row.token_address  {}", repair_row.token_address);

            let (recent_blockhash, _) = client.get_recent_blockhash().unwrap();

            let program_id = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
                .parse()
                .unwrap();

            let instruction = update_metadata_accounts(
                program_id,
                *metadata_address,
                metadata.update_authority,
                Some(new_update_authority),
                None,
                None,
            );

            let instructions = &[instruction];

            let signing_keypairs = &[&current_update_authority];

            let tx = Transaction::new_signed_with_payer(
                instructions,
                Some(&current_update_authority.pubkey()),
                signing_keypairs,
                recent_blockhash,
            );

            // let res = client.simulate_transaction(&tx);
            // let res = res.expect("could not simulate tx");
            // eprintln!("{:?}", res);

            let res = client.send_and_confirm_transaction(&tx);
            let sig = res.expect("could not confirm tx");
            eprintln!("{:?}", sig);
        }
    }

    Ok(())
}

fn repair_metabaes(app_options: AppOptions, opts: RepairMetabaesArgs) -> Result<()> {
    let client = RpcClient::new(app_options.rpc_url);
    let db = Connection::open(app_options.db_path)?;

    let mut stmt = db.prepare(
        "SELECT token_address, metadata_address, old_name, new_name, old_uri, new_uri FROM repairs ORDER BY token_address",
    )?;

    let repair_row_iter = stmt.query_map([], |row| {
        Ok(RepairRow {
            token_address: row.get(0)?,
            metadata_address: row.get(1)?,
            old_name: row.get(2)?,
            new_name: row.get(3)?,
            old_uri: row.get(4)?,
            new_uri: row.get(5)?,
        })
    })?;

    for repair_row in repair_row_iter {
        let repair_row = repair_row.unwrap();
        eprintln!("repair_row.token_address  {}", repair_row.token_address);

        let metadata_address = &repair_row
            .metadata_address
            .parse()
            .expect("could not parse metadata_address");

        let account = client
            .get_account(metadata_address)
            .expect("could not fetch metadata account");

        let mut buf = account.data();
        let metadata = Metadata::deserialize(&mut buf).expect("could not deserialize metadata");

        if {
            repair_row.new_name != metadata.data.name.to_string().trim_matches(char::from(0))
                || repair_row.new_uri != metadata.data.uri.to_string().trim_matches(char::from(0))
        } {
            eprintln!(
                "need to repair: token {}, metadata {}",
                repair_row.token_address, repair_row.metadata_address
            );
            eprintln!(
                "  {} => {}",
                metadata.data.name.to_string().trim_matches(char::from(0)),
                repair_row.new_name,
            );
            eprintln!(
                "  {} => {}",
                metadata.data.uri.to_string().trim_matches(char::from(0)),
                repair_row.new_uri,
            );

            let (recent_blockhash, _) = client.get_recent_blockhash().unwrap();

            let program_id = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
                .parse()
                .unwrap();

            let instruction = update_metadata_accounts(
                program_id,
                *metadata_address,
                metadata.update_authority,
                None,
                Some(Data {
                    name: repair_row.new_name,
                    uri: repair_row.new_uri,
                    symbol: metadata.data.symbol,
                    seller_fee_basis_points: metadata.data.seller_fee_basis_points,
                    creators: metadata.data.creators,
                }),
                None,
            );

            let instructions = &[instruction];

            let keypair =
                read_keypair_file(opts.keypair_path.clone()).expect("could not read keypair file");

            let signing_keypairs = &[&keypair];

            let tx = Transaction::new_signed_with_payer(
                instructions,
                Some(&keypair.pubkey()),
                signing_keypairs,
                recent_blockhash,
            );

            // let res = client.simulate_transaction(&tx);
            // let res = res.expect("could not simulate tx");
            // eprintln!("{:?}", res);

            let res = client.send_and_confirm_transaction(&tx);
            let sig = res.expect("could not confirm tx");
            eprintln!("{:?}", sig);
        }
    }

    // db.execute(
    //     "create table if not exists repairs (
    //         token_address text primary key,
    //         metadata_address text unique,
    //         old_name text,
    //         new_name text,
    //         old_uri text,
    //         new_uri text
    //     )",
    //     params![],
    // )?;

    // let mut stmt = db.prepare("SELECT token_address, metadata_address, key, update_authority, mint, name, symbol, uri, seller_fee_basis_points, primary_sale_happened, is_mutable, edition_nonce FROM metadatas where name like ?1")?;

    // for n in 0..1000 {
    //     let name = format!("Metabaes #{}", n);

    //     let mut metadata_row_iter = stmt.query_map(params![name], |row| {
    //         Ok(MetadataRow {
    //             token_address: row.get(0)?,
    //             metadata_address: row.get(1)?,
    //             key: row.get(2)?,
    //             update_authority: row.get(3)?,
    //             mint: row.get(4)?,
    //             name: row.get(5)?,
    //             symbol: row.get(6)?,
    //             uri: row.get(7)?,
    //             seller_fee_basis_points: row.get(8)?,
    //             primary_sale_happened: row.get(9)?,
    //             is_mutable: row.get(10)?,
    //             edition_nonce: row.get(11)?,
    //         })
    //     })?;

    //     let _ = metadata_row_iter.next().unwrap().unwrap();
    //     let sad_bae = metadata_row_iter.next().unwrap().unwrap();

    //     // todo skip if sadBae in repairs
    //     let count: Result<u8, rusqlite::Error> = db.query_row(
    //         "select count(*) from repairs where metadata_address = ?1",
    //         params![sad_bae.metadata_address.to_string()],
    //         |row| row.get(0),
    //     );
    //     if count.unwrap() >= 1u8 {
    //         continue;
    //     }

    //     db.execute(
    //         "INSERT INTO repairs
    //         (token_address, metadata_address, old_name, new_name, old_uri) values
    //         (           ?1,               ?2,       ?3,       ?4,      ?5)",
    //         params![
    //             sad_bae.token_address.to_string(),
    //             sad_bae.metadata_address.to_string(),
    //             sad_bae.name.to_string(),
    //             format!("Metabaes #{}", (n + 7888)),
    //             sad_bae.uri.to_string(),
    //         ],
    //     )?;
    // }

    // TODO
    // - for each repair where old_uri is null
    // - fetch the metadata json from arweave
    // - correct the name property
    // - upload the updated json to arweave
    // - store the new_uri

    // TODO
    // - for each repair
    // - fetch the metadata account from Solana
    // - if the name or uri isn't correct
    // - submit a UpdateMetadata instruction

    Ok(())
}

fn mine_tokens_by_update_authority(
    app_options: AppOptions,
    opts: ByUpdateAuthorityArgs,
) -> Result<()> {
    let client = RpcClient::new(app_options.rpc_url);
    let db = Connection::open(app_options.db_path)?;

    db.execute(
        "create table if not exists tokens (
             token_address text primary key,
             metadata_address text unique,
             genesis_signature text unique,
             genesis_block_time numeric
         )",
        params![],
    )?;

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
        let count: Result<u8, rusqlite::Error> = db.query_row(
            "select count(*) from tokens where metadata_address = ?1",
            params![metadata_address.to_string()],
            |row| row.get(0),
        );

        let count = count.unwrap();
        if count >= 1u8 {
            eprint!("{}", "-");
            continue;
        } else {
            eprint!("{}", "+");
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

        let genesis_signature = sigs.last().unwrap();
        let genesis_block_time = genesis_signature.block_time.unwrap();

        let genesis_signature = genesis_signature.signature.parse().unwrap();

        let tx = client.get_transaction(&genesis_signature, UiTransactionEncoding::Base58);
        if let Err(err) = tx {
            eprintln!("\ncouldn't get transaction {} {}", genesis_signature, err);
            continue;
        }

        let tx = tx.unwrap().transaction;
        let tx = tx.transaction.decode();
        if let None = tx {
            eprintln!("\ncould not decode sig tx {} {}", pubkey, genesis_signature);
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
            eprintln!("\ncouldn't get token address {}", genesis_signature);
            continue;
        }

        let token_address = token_address.unwrap();

        db.execute(
            "INSERT INTO tokens
            (token_address, metadata_address, genesis_signature, genesis_block_time) values
            (?1           , ?2              , ?3               , ?4               )",
            params![
                token_address.to_string(),
                metadata_address.to_string(),
                genesis_signature.to_string(),
                genesis_block_time,
            ],
        )?;
    }

    Ok(())
}

fn mine_token_metadata(app_options: AppOptions, _opts: MineTokenMetadataArgs) -> Result<()> {
    let client = RpcClient::new(app_options.rpc_url);
    let db = Connection::open(app_options.db_path).expect("could not open db");

    db.execute(
        "create table if not exists metadatas (
            token_address text primary key,
            metadata_address text unique,
            key text,
            update_authority text,
            mint text,
            name text,
            symbol text,
            uri text,
            seller_fee_basis_points numeric,
            primary_sale_happened integer,
            is_mutable integer,
            edition_nonce integer
        )",
        params![],
    )?;

    db.execute(
        "create table if not exists creators (
            metadata_address text,
            address text,
            share numeric,
            idx numeric
        )",
        params![],
    )?;

    let mut stmt = db.prepare("SELECT token_address, metadata_address, genesis_signature, genesis_block_time FROM tokens order by genesis_block_time, token_address;")?;
    let token_row_iter = stmt.query_map([], |row| {
        Ok(TokenRow {
            token_address: row.get(0)?,
            metadata_address: row.get(1)?,
            genesis_signature: row.get(2)?,
            genesis_block_time: row.get(3)?,
        })
    })?;

    for token_row in token_row_iter {
        let token_row = token_row.unwrap();

        let count: Result<u8, rusqlite::Error> = db.query_row(
            "select count(*) from metadatas where metadata_address = ?1",
            params![token_row.metadata_address.to_string()],
            |row| row.get(0),
        );

        let count = count.unwrap();
        if count >= 1u8 {
            eprint!("{}", "-");
            continue;
        } else {
            eprint!("{}", "+");
        }

        let metadata_address = &token_row
            .metadata_address
            .parse()
            .expect("could not parse metadata_address");

        let account = client
            .get_account(metadata_address)
            .expect("could not fetch metadata account");

        let mut buf = account.data();
        let metadata = Metadata::deserialize(&mut buf).expect("could not deserialize metadata");

        db.execute(
            "INSERT INTO metadatas
            (token_address, metadata_address, key, update_authority, mint, name, symbol, uri, seller_fee_basis_points, primary_sale_happened, is_mutable, edition_nonce) values
            (?1           , ?2              , ?3 , ?4              , ?5  , ?6  , ?7    , $8 , $9                     , $10                  , $11       , $12          )",
            params![
                token_row.token_address.to_string(),
                metadata_address.to_string(),
                format!("{:?}",metadata.key),
                metadata.update_authority.to_string(),
                metadata.mint.to_string(),
                metadata.data.name.to_string().trim_matches(char::from(0)),
                metadata.data.symbol.to_string().trim_matches(char::from(0)),
                metadata.data.uri.to_string().trim_matches(char::from(0)),
                metadata.data.seller_fee_basis_points,
                metadata.primary_sale_happened,
                metadata.is_mutable,
                metadata.edition_nonce,
            ],
        )?;

        if let Some(creators) = metadata.data.creators {
            let mut idx = 0u8;
            for creator in creators {
                idx = idx + 1u8;
                db.execute(
                    "INSERT INTO creators (metadata_address, address, share, idx) values (?1, ?2, ?3, ?4)",
                    params![
                        metadata_address.to_string(),
                        creator.address.to_string(),
                        creator.share,
                        idx,
                    ],
                )?;
            }
        }
    }

    Ok(())
}

fn default_rpc_url() -> String {
    "https://api.mainnet-beta.solana.com".to_owned()
}

fn default_db_path() -> String {
    "candy-holders.db".to_owned()
}
