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
          sha256 = "sha256-ggvRZZFjlAlrZVjqul/f/UpU5CEhDbdKZU0OCR8Uzbc=";
        };
        buildInputs = with pkgs; [
          pkg-config
          rustToolchain
          rust-analyzer
          cargo-expand
          probe-rs-tools
          libusb1
          libusb1.dev
          libclang
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
