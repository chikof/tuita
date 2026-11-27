# Kind-specific devenv settings, merged into devenv.nix via `imports`.
{pkgs, ...}: {
  services.postgres = {
    enable = true;
    package = pkgs.postgresql_18;
    listen_addresses = "127.0.0.1";
    port = 5432;
    # Parallel test binaries, each with its own pools, run out of the default 100 connections.
    settings.max_connections = 500;
    # Every `#[sqlx::test]` creates and drops a database, and each of those forces an immediate checkpoint that
    # all other tests wait behind. A dev database may lose its data in a crash, so skip the durability work.
    settings.fsync = "off";
    settings.synchronous_commit = "off";
    settings.full_page_writes = "off";
    # CREATEDB: #[sqlx::test] creates one database per test.
    initialScript = "CREATE ROLE tuita LOGIN CREATEDB;";
    initialDatabases = [
      {
        name = "tuita";
        user = "tuita";
      }
    ];
  };

  env.DATABASE_URL = "postgres://tuita@localhost/tuita";

  # `devenv up` runs Postgres and the server. The server needs the asset bundle next to its binary.
  processes.tuita.exec = "topcoat asset bundle --package tuita && cargo run -p tuita -- serve";
}
