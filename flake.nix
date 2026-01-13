# SPDX-License-Identifier: Unlicense
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachSystem nixpkgs.lib.systems.flakeExposed (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        desktop = import ./nix/sermo.nix {
          package-type = "desktop";
          inherit pkgs rustBuildInputs rustToolchain;
        };

        web = import ./nix/sermo.nix {
          package-type = "web";
          inherit pkgs rustBuildInputs rustToolchain;
        };

        mobile = import ./nix/sermo.nix {
          package-type = "mobile";
          inherit pkgs rustBuildInputs rustToolchain;
        };

        # ref: "https://github.com/DioxusLabs/dioxus/blob/main/flake.nix"
        rustBuildInputs =
          with pkgs;
          [
            openssl
            libiconv
            pkg-config
          ]
          ++ lib.optionals pkgs.stdenv.isLinux [
            glib
            gtk3
            libsoup_3
            libudev-zero
            webkitgtk_4_1
            xdotool
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            apple-sdk
            libiconv
          ];

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        packages = flake-utils.lib.flattenTree {
          inherit desktop web mobile;
          default = desktop;
        };

        devShells.default = pkgs.mkShell {
          packages =
            with pkgs;
            [
              rustToolchain
              wasm-bindgen-cli
              dioxus-cli
            ]
            ++ rustBuildInputs;

          # Resolve dioxus-desktop compilation issue: "https://github.com/DioxusLabs/dioxus/issues/5203"
          shellHook = ''
            cargo update -p zbus --precise 5.5.0
            cargo update -p zbus_macros --precise 5.5.0
            cargo update -p zbus_names --precise 4.2.0  # requires zvariant ^5.9.0 in 4.3.1
            cargo update -p zvariant --precise 5.8.0
            cargo update -p zvariant_derive --precise 5.8.0
          '';
        };
        formatter = treefmt-nix.lib.mkWrapper pkgs {
          projectRootFile = "flake.nix";
          programs = {
            nixfmt.enable = true;
            rustfmt.enable = true; # Rust
            taplo.enable = true; # TOML
            prettier.enable = true; # JS/TS/JSON/MD/...
          };
        };
      }
    );
}
