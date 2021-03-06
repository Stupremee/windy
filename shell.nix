{ pkgs ? import <nixpkgs> { } }:
let
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;

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
        -d guest_errors,trace:riscv_trap,trace:sifive_gpio_write,trace:pmpcfg_csr_write,trace:pmpaddr_csr_write,int,trace:exynos_uart_read \
        -bios ${opensbi}/platform/fw_dynamic.bin \
        -gdb tcp::1234 \
        $QEMUFLAGS \
        -kernel "$@"
  '';
in pkgs.mkShell {
  name = "rust-shell";
  nativeBuildInputs = with pkgs; [
    rust
    llvm_11
    qemu
    python3
    dtc
    cargo-expand
    runQemu

    genext2fs
  ];
}
