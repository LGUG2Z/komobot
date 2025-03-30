{pkgs ? import (fetchTarball "https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz") {}}:
with pkgs;
  mkShell {
    name = "komobot";

    buildInputs = [
      alejandra
      bacon
      just
      libiconv
      openssl
      pkg-config
      rustup
    ];
  }
