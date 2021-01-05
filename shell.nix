{ pkgs ? import <nixpkgs> { } }:
let
  rustChannel = pkgs.rustChannelOf {
    channel = "nightly";
    date = "2020-12-22";
  };

  rust = rustChannel.rust.override {
    extensions = [ "rust-src" ];
    targets = [ "riscv64gc-unknown-none-elf" ];
  };

  pkgsRiscv =
    import <nixpkgs> { crossSystem = { config = "riscv64-none-elf"; }; };
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
        -bios ${opensbi}/platform/fw_dynamic.elf \
        -gdb tcp::1234 \
        -kernel "$@"
  '';
in pkgs.mkShell {
  name = "rust-shell";
  nativeBuildInputs = with pkgs;
    [ rust-analyzer rust llvm_11 qemu python3 cargo-expand ] ++ [ runQemu ];
}
