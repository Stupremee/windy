{
  description = "Operating System for RISC-V written in Rust.";

  inputs = {
    devshell.url = "github:numtide/devshell";
    naersk.url = "github:nmattia/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      flake = false;
    };
  };

  outputs =
    { nixpkgs, rust-overlay, naersk, gitignore, devshell, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        version = "0.1.0";
        pname = "windy";

        inherit (gitignore-lib) gitignoreSource;
        inherit (flake-utils.lib) mkApp;

        gitignore-lib = import gitignore { inherit (pkgs) lib; };
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlay rust-overlay.overlay ];
        };

        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in rec {
        # `nix build`
        packages."${pname}" = naersk-lib.buildPackage {
          inherit version;
          root = gitignoreSource ./.;
        };
        defaultPackage = packages."${pname}";

        # `nix run`
        apps."${pname}" = mkApp {
          name = pname;
          drv = packages."${pname}";
        };
        defaultApp = apps."${pname}";

        # `nix develop`
        devShell = pkgs.devshell.mkShell {
          name = pname;
          packages = with pkgs; [ rust ];
        };
      });
}
