{
  description = "rsmetrx - small Rust utilities for system metrics";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in rec {
        # Build the main binary from Cargo.toml/Cargo.lock in repo root
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rsmetrx";
          version = "0.1.0";
          src = ./.;

          # Use the checked-in lockfile for reproducible builds
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Add native deps here if needed (e.g. pkg-config)
          nativeBuildInputs = [ ];
          buildInputs = [ ];

          # Optional metadata
          meta = with pkgs.lib; {
            description = "System metrics CLI utilities";
            homepage = "https://github.com/neg-serg/rsmetrx";
            license = licenses.mit;
            maintainers = [ maintainers.undefined ];
            mainProgram = "rsmetrx";
          };
        };

        # Enable `nix run .#rsmetrx`
        apps.default = {
          type = "app";
          program = "${packages.default}/bin/rsmetrx";
        };

        # Dev shell with common Rust tooling
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
          ];
        };
      }
    );
}
