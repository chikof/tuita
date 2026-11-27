//! `tuita` command line: the server and its maintenance commands.

use anyhow::{Context as _, Result};
use clap::{Parser, Subcommand};
use tuita_server::admin::{self, Action};
use tuita_server::federation::keys;
use tuita_server::media::MediaStore;
use tuita_server::seed::{self, Outcome};
use tuita_server::{AppState, Config, connect, init_tracing, migrate, serve, shutdown_signal};

/// tuita: a small social platform with a 1-bit dither look that federates over ActivityPub.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run migrations, then serve HTTP until SIGINT/SIGTERM (the default).
    Serve,
    /// Run pending database migrations and exit.
    Migrate,
    /// Manage accounts from the shell.
    Admin {
        #[command(subcommand)]
        action: AdminAction,
    },
    /// Fill an empty database with demo users and posts for development.
    Seed,
}

#[derive(Debug, Subcommand)]
enum AdminAction {
    /// Make an account an admin.
    Promote { handle: Box<str> },
    /// Make an account a moderator.
    Moderator { handle: Box<str> },
    /// Make an account a regular user again.
    Demote { handle: Box<str> },
    /// Give an account the verified badge.
    Verify { handle: Box<str> },
    /// Remove the verified badge.
    Unverify { handle: Box<str> },
    /// Suspend an account.
    Suspend { handle: Box<str> },
    /// Lift a suspension.
    Unsuspend { handle: Box<str> },
    /// List admins, moderators and verified accounts.
    List,
}

impl AdminAction {
    /// The account command and its handle; `None` for `list`.
    fn account(&self) -> Option<(Action, &str)> {
        match self {
            Self::Promote { handle } => Some((Action::Promote, handle)),
            Self::Moderator { handle } => Some((Action::Moderator, handle)),
            Self::Demote { handle } => Some((Action::Demote, handle)),
            Self::Verify { handle } => Some((Action::Verify, handle)),
            Self::Unverify { handle } => Some((Action::Unverify, handle)),
            Self::Suspend { handle } => Some((Action::Suspend, handle)),
            Self::Unsuspend { handle } => Some((Action::Unsuspend, handle)),
            Self::List => None,
        }
    }
}

/// Stack of each runtime worker thread. Rendering a page nests many large futures; the 2 MiB default
/// overflows in debug builds.
const WORKER_STACK: usize = 16 * 1024 * 1024;

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(WORKER_STACK)
        .build()
        .context("cannot start the async runtime")?
        .block_on(run())
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::from_env()?;
    init_tracing(config.log_format);
    let db = connect(&config).await?;
    migrate(&db).await?;
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => {
            let media = MediaStore::open(&config).await?;
            tracing::info!(storage = ?media, "media storage ready");
            keys::protect_at_rest(&db, &config).await?;
            let assets = tuita_web::Assets::next_to_binary().context(
                "asset bundle missing next to the binary; run `topcoat asset bundle --package tuita`",
            )?;
            let state = AppState::new(db, config, media);
            serve(
                state.clone(),
                tuita_web::router(state, assets),
                shutdown_signal(),
            )
            .await
        }
        Command::Migrate => {
            tracing::info!("migrations applied");
            Ok(())
        }
        Command::Admin { action } => {
            if let Some((command, handle)) = action.account() {
                admin::apply(&db, command, handle).await?;
                let handle = handle.strip_prefix('@').unwrap_or(handle);
                println!("Done: {} @{handle}", command.name());
            } else {
                let rows = admin::list(&db).await.context("cannot list staff")?;
                println!(
                    "{:<16} {:<10} {:<9} suspended",
                    "handle", "role", "verified"
                );
                for row in rows {
                    println!(
                        "{:<16} {:<10} {:<9} {}",
                        row.handle, row.role, row.verified, row.suspended
                    );
                }
            }
            Ok(())
        }
        Command::Seed => {
            match seed::run(&db).await.context("cannot seed the database")? {
                Outcome::Seeded { users, posts } => {
                    println!("Seeded {users} accounts and {posts} posts.");
                }
                Outcome::AlreadyPopulated => {
                    println!("The database already has accounts; nothing seeded.");
                }
            }
            Ok(())
        }
    }
}
