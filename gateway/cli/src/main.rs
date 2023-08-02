use bitcoin::{Address, Amount};
use clap::{CommandFactory, Parser, Subcommand};
use fedimint_core::config::FederationId;
use fedimint_logging::TracingSetup;
use ln_gateway::rpc::rpc_client::GatewayRpcClient;
use ln_gateway::rpc::{
    BackupPayload, BalancePayload, ConnectFedPayload, DepositAddressPayload, RestorePayload,
    WithdrawPayload,
};
use serde::Serialize;
use url::Url;

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// The address of the gateway webserver
    #[clap(short, long, default_value = "http://127.0.0.1:8175")]
    address: Url,
    #[command(subcommand)]
    command: Commands,
    /// WARNING: Passing in a password from the command line may be less secure!
    #[clap(long)]
    rpcpassword: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Display CLI version hash
    VersionHash,
    /// Display high-level information about the Gateway
    Info,
    /// Check gateway balance
    Balance {
        #[clap(long)]
        federation_id: FederationId,
    },
    /// Generate a new peg-in address, funds sent to it can later be claimed
    Address {
        #[clap(long)]
        federation_id: FederationId,
    },
    /// Claim funds from a gateway federation
    Withdraw {
        #[clap(long)]
        federation_id: FederationId,
        /// The amount to withdraw
        #[clap(long)]
        amount: Amount,
        /// The address to send the funds to
        #[clap(long)]
        address: Address,
    },
    /// Register federation with the gateway
    ConnectFed {
        /// InviteCode code to connect to the federation
        invite_code: String,
    },
    /// Make a backup of snapshot of all ecash
    Backup {
        #[clap(long)]
        federation_id: FederationId,
    },
    /// Restore ecash from last available snapshot or from scratch
    Restore {
        #[clap(long)]
        federation_id: FederationId,
    },
    Completion {
        shell: clap_complete::Shell,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TracingSetup::default().init()?;

    let cli = Cli::parse();
    let client = || GatewayRpcClient::new(cli.address, source_password(cli.rpcpassword));

    match cli.command {
        Commands::VersionHash => {
            println!("{}", env!("FEDIMINT_BUILD_CODE_VERSION"));
        }
        Commands::Info => {
            let response = client().get_info().await?;

            print_response(response).await;
        }
        Commands::Balance { federation_id } => {
            let response = client()
                .get_balance(BalancePayload { federation_id })
                .await?;

            print_response(response).await;
        }
        Commands::Address { federation_id } => {
            let response = client()
                .get_deposit_address(DepositAddressPayload { federation_id })
                .await?;

            print_response(response).await;
        }
        Commands::Withdraw {
            federation_id,
            amount,
            address,
        } => {
            let response = client()
                .withdraw(WithdrawPayload {
                    federation_id,
                    amount,
                    address,
                })
                .await?;

            print_response(response).await;
        }
        Commands::ConnectFed { invite_code } => {
            let response = client()
                .connect_federation(ConnectFedPayload { invite_code })
                .await?;

            print_response(response).await;
        }
        Commands::Backup { federation_id } => {
            client().backup(BackupPayload { federation_id }).await?;
        }
        Commands::Restore { federation_id } => {
            client().restore(RestorePayload { federation_id }).await?;
        }
        Commands::Completion { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "gateway-cli",
                &mut std::io::stdout(),
            );
        }
    }

    Ok(())
}

pub async fn print_response<T: Serialize>(val: T) {
    println!(
        "{}",
        serde_json::to_string_pretty(&val).expect("Cannot serialize")
    )
}

pub fn source_password(rpcpassword: Option<String>) -> String {
    match rpcpassword {
        None => rpassword::prompt_password("Enter gateway password:").unwrap(),
        Some(password) => {
            eprintln!("WARNING: Passing in a password from the command line may be less secure!");
            password
        }
    }
}
