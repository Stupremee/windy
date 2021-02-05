{
  description = "Operating System for RISC-V written in Rust.";

  inputs = {
    devshell.url = "github:numtide/devshell";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, devshell, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        inherit (flake-utils.lib) mkApp;

        pkgsRiscv = import nixpkgs {
          inherit system;
          crossSystem = { config = "riscv64-none-elf"; };
        };

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlay rust-overlay.overlay ];
        };

        opensbi = pkgsRiscv.callPackage ./nix/opensbi.nix { };
        runQemu = pkgs.writers.writeBashBin "run-qemu" ''
          ${pkgs.qemu}/bin/qemu-system-riscv64 \
              -machine virt \
              -cpu rv64 \
              -smp 4 \
              -m 128M \
              -nographic \
              -serial mon:stdio \
              -d guest_errors,trace:riscv_trap,trace:sifive_gpio_write,trace:pmpcfg_csr_write,trace:pmpaddr_csr_write,int,trace:exynos_uart_read \
              -bios ${opensbi}/platform/fw_dynamic.bin \
              -gdb tcp::1234 \
              -kernel "$@"
        '';
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
      in {
        # `nix run qemu`
        apps.qemu = mkApp {
          name = "run-qemu";
          drv = runQemu;
        };

        # `nix develop`
        devShell = pkgs.mkShell {
          name = "windy-development";
          nativeBuildInputs = [ rust runQemu ];
          #packages = [ rust runQemu ];
          #commands = [{
          #name = "run-qemu";
          #help = "Cargo runner for running Windy inside QEMU";
          #package = runQemu;
          #}];
        };
      });
}
