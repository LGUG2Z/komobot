{
  description = "komobot";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    git-hooks-nix.url = "github:cachix/git-hooks.nix";
    git-hooks-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      crane,
      rust-overlay,
      ...
    }:
    let
      mkKomobotPackages =
        { pkgs }:
        let
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
          version = "0.1.0";

          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type: (craneLib.filterCargoSources path type);
          };

          commonArgs = {
            inherit src version;
            strictDeps = true;
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs = with pkgs; [
              openssl
            ];
            doCheck = false;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          individualCrateArgs = commonArgs // {
            inherit cargoArtifacts;
            doCheck = false;
            doDoc = false;
          };

          komobot = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "komobot";
            }
          );
        in
        {
          inherit
            craneLib
            src
            individualCrateArgs
            komobot
            ;
        };

      mkPkgs =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
      ];

      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.git-hooks-nix.flakeModule
      ];

      perSystem =
        { config, system, ... }:
        let
          pkgs = mkPkgs system;
          build = mkKomobotPackages { inherit pkgs; };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          nightlyRustfmt = pkgs.rust-bin.nightly.latest.rustfmt;
          rustToolchainWithNightlyRustfmt = pkgs.symlinkJoin {
            name = "rust-toolchain-with-nightly-rustfmt";
            paths = [
              nightlyRustfmt
              rustToolchain
            ];
          };
        in
        {
          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              deadnix.enable = true;
              just.enable = true;
              nixfmt.enable = true;
              taplo.enable = true;
              rustfmt = {
                enable = true;
                package = pkgs.rust-bin.nightly.latest.rustfmt;
              };
            };
          };

          checks = {
            komobot-clippy = build.craneLib.cargoClippy build.individualCrateArgs;

            komobot-fmt = build.craneLib.cargoFmt {
              inherit (build) src;
            };

            komobot-toml-fmt = build.craneLib.taploFmt {
              src = pkgs.lib.sources.sourceFilesBySuffices build.src [ ".toml" ];
            };

            komobot-deny = build.craneLib.cargoDeny {
              inherit (build) src;
            };
          };

          packages = {
            inherit (build) komobot;
            default = build.komobot;
          };

          apps = {
            komobot = {
              type = "app";
              program = "${build.komobot}/bin/komobot";
            };
            default = {
              type = "app";
              program = "${build.komobot}/bin/komobot";
            };
          };

          devShells.default = pkgs.mkShell {
            name = "komobot";

            inputsFrom = [ build.komobot ];

            packages = [
              rustToolchainWithNightlyRustfmt

              pkgs.bacon
              pkgs.cargo-deny
              pkgs.just
            ];
          };

          pre-commit = {
            check.enable = true;
            settings.hooks.treefmt = {
              enable = true;
              package = config.treefmt.build.wrapper;
              pass_filenames = false;
            };
          };
        };
    };
}
