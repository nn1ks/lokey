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
          sha256 = "sha256-sfz/L5xfR+ltvPiC4TnCUhsBjzi7I25Mj7DC8YyNATM=";
        };
        buildInputs = with pkgs; [
          pkg-config
          rustToolchain
          rust-analyzer
          probe-rs
          libusb1
          libusb1.dev
        ];
      in {
        # `nix develop`
        devShell = pkgs.mkShell {
          inherit buildInputs;
        };
      }
    );
}
