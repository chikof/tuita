{pkgs, ...}: {
  imports = [./devenv.kind.nix];

  languages.rust = {
    enable = true;
    # rustup reads the same file outside Nix.
    toolchainFile = ./rust-toolchain.toml;
  };

  packages = [
    pkgs.git
    pkgs.shellcheck
    pkgs.typos
    pkgs.cargo-deny
    pkgs.cargo-nextest
    pkgs.bacon
    pkgs.git-cliff # CHANGELOG.md, scripts/release.sh
    pkgs.cargo-edit # cargo set-version, scripts/release.sh
    pkgs.sqlx-cli
    pkgs.openssl
    pkgs.pkg-config
    pkgs.ripgrep # scripts/lint-strings.sh
    pkgs.chromium # scripts/e2e.sh (crates/tuita-e2e drives it over CDP)
  ];

  env.E2E_CHROME = "${pkgs.chromium}/bin/chromium";

  # Secrets and local overrides come from the gitignored .env file. See .env.example.
  dotenv.enable = true;

  # scripts/check.sh is the single gate.
  scripts.check.exec = ''
    exec "$DEVENV_ROOT/scripts/check.sh"
  '';

  # topcoat-cli (asset bundler, fmt, l10n check) is pinned to the topcoat crate version and installed once per devenv state.
  enterShell = ''
    [ -x "$DEVENV_STATE/topcoat-cli-0.12.1/bin/topcoat" ] \
      || cargo install --locked --version 0.12.1 --root "$DEVENV_STATE/topcoat-cli-0.12.1" topcoat-cli
    export PATH="$DEVENV_STATE/topcoat-cli-0.12.1/bin:$PATH"
    echo "tuita: run 'check'"
  '';

  # rustfmt and clippy are not hooks on purpose: `check` covers them and commits stay fast.
  git-hooks.hooks = {
    convco.enable = true; # commit-msg: Conventional Commits
    ripsecrets.enable = true;
    typos.enable = true;
    shellcheck = {
      enable = true;
      excludes = ["^\\.envrc$"];
    };
  };
}
