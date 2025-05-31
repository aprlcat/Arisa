{
  description = "Arisa";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        commonBuildInputs = with pkgs; [
          pkg-config
          openssl
        ];

        platformBuildInputs = with pkgs; lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "arisa";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = commonBuildInputs;
          buildInputs = commonBuildInputs ++ platformBuildInputs;

          env = {
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OPENSSL_NO_VENDOR = "1";
          };

          doCheck = false;

          meta = with pkgs.lib; {
            description = "Discord bot for nerds, by nerds";
            license = licenses.mit;
            platforms = platforms.unix;
            mainProgram = "arisa";
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];

          buildInputs = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-edit
            rust-analyzer
          ] ++ commonBuildInputs ++ platformBuildInputs;

          env = {
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            RUST_LOG = "debug";
            OPENSSL_NO_VENDOR = "1";
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          exePath = "/bin/arisa";
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
