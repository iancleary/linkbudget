{
  description = "A Nix-flake-based Rust development environment";

  # GitHub URLs for the Nix inputs we're using
  inputs = {
    # Simply the greatest package repository on the planet
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # A set of helper functions for using flakes
    flake-utils.url = "github:numtide/flake-utils";
    # A utility library for working with Rust
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          # This overlay adds the "rust-bin" package to nixpkgs
          (import rust-overlay)
        ];

        # System-specific nixpkgs with rust-overlay applied
        pkgs = import nixpkgs { inherit system overlays; };

        # Use the specific version of the Rust toolchain specified by the toolchain file
        localRust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Cargo subcommands commonly used in Rust projects
        cargoSubCommands = with pkgs; [
          cargo-audit
          cargo-edit
          cargo-release
          cargo-watch
        ];

        # Placeholder for utilities commonly used in Rust projects (but not in this example project)
        others = with pkgs; [ just ];

      in {
        devShells = {
          default = pkgs.mkShell {
            # Packages included in the environment
            buildInputs = [ localRust ] ++ cargoSubCommands ++ others;

            # Run when the shell is started up
            shellHook = ''
              ${localRust}/bin/cargo --version
            '';
          };
        };
      });
}
