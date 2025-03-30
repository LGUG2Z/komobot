{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          overlays = [
            (import rust-overlay)
          ];
          pkgs = (import nixpkgs) {
            inherit system overlays;
          };

          inherit (pkgs) lib;

          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

          pname = "komobot";
          version = "0.1.0";

          commonArgs = {
            inherit pname version;
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            doCheck = false;
            buildInputs = with pkgs; [
              openssl
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly (commonArgs
            // {
              src = craneLib.cleanCargoSource (craneLib.path ./.);
            });

          komobot = craneLib.buildPackage (commonArgs
            // {
              inherit cargoArtifacts;
              src = lib.cleanSourceWith {
                src = ./.;
              };
            });
        in {
          devShells = flake-utils.lib.flattenTree {
            default = import ./shell.nix {inherit pkgs;};
          };

          packages = flake-utils.lib.flattenTree rec {
            inherit komobot;
            default = komobot;
          };
        }
      );
}
