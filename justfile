reload:
    direnv reload

run:
    cargo +stable run

fmt:
    cargo +nightly fmt
    cargo clippy
    alejandra flake.nix shell.nix

build:
    nix build . && attic push system result
