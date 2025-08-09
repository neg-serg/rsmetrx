{
  description = "Zsh full build environment (with ncurses, Rust, etc.)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.zsh
            pkgs.pkg-config
            pkgs.tree-sitter
            pkgs.lscolors
            # Full C build env for zsh:
            pkgs.gcc
            pkgs.gnumake
            pkgs.autoconf
            pkgs.automake
            pkgs.perl
            pkgs.patch
            pkgs.ncurses
            pkgs.ncurses.dev
            pkgs.readline
            pkgs.readline.dev
            pkgs.libcap
            pkgs.libcap.dev
            pkgs.bison
            pkgs.flex
            pkgs.gettext
            pkgs.util-linux # for 'script' tool etc.
          ];
          shellHook = ''
            echo "Zsh build environment ready!"
          '';
        };
      }
    );
}

