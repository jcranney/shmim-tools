{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      rec {
        defaultPackage = naersk-lib.buildPackage ./.;
        packages.default = defaultPackage;
        devShell = with pkgs; mkShell rec {
          buildInputs = [
            cmake cargo rustc rustfmt pre-commit
            pkg-config rustfmt rust-analyzer
            cargo-watch clippy cargo-machete
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}
