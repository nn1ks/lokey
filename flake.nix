{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, fenix }:
    utils.lib.eachDefaultSystem(system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-iq96Kl8mFhDlI47uYI7LAcP8uf6keQeBYksiLE7zC54=";
        };
        buildInputs = with pkgs; [
          pkg-config
          rustToolchain
          rust-analyzer
          probe-rs-tools
          libusb1
          libusb1.dev
          libclang

          # Tools
          cargo-release
          cargo-expand
          cargo-binutils
          cargo-bloat
          cargo-outdated
          cargo-depgraph
          graphviz

          # Docs website
          nodejs
        ];
      in {
        # `nix develop`
        devShell = pkgs.mkShell {
          inherit buildInputs;
          shellHook = ''
            export LIBCLANG_PATH=${pkgs.libclang.lib}/lib
          '';
        };
      }
    );
}
