{
  package-type,
  pkgs,
  rustBuildInputs,
  rustToolchain ? pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml,
  ...
}:

let
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "sermo-${package-type}";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  buildInputs = rustBuildInputs;
  nativeBuildInputs = with pkgs; [
    rustToolchain
    pkg-config
    dioxus-cli
    wasm-bindgen-cli
  ];

  # Resolve dioxus-desktop compilation issue: "https://github.com/DioxusLabs/dioxus/issues/5203"
  patchPhase = ''
    cargo update -p zbus --precise 5.5.0
    cargo update -p zbus_macros --precise 5.5.0
    cargo update -p zbus_names --precise 4.2.0  # requires zvariant ^5.9.0 in 4.3.1
    cargo update -p zvariant --precise 5.8.0
    cargo update -p zvariant_derive --precise 5.8.0
  '';
  buildPhase = ''
    cargo build --release --bin ${package-type}
  '';
  installPhase = ''
    mkdir -p $out/bin
    cp target/release/${package-type} $out/bin/sermo-${package-type}
  '';
  doCheck = false; # Disable tests to avoid building deps for them
}
