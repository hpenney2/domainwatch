{
  pkgs ? import <nixpkgs> { },
}:
pkgs.rustPlatform.buildRustPackage (finalAttrs: {
  pname = "iphotokam";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;

  nativeBuildInputs = with pkgs; [
    rustPlatform.bindgenHook
    pkg-config
    openssl
  ];
})
