{ pkgs ? import <nixpkgs> { } }:
let
  rustChannel = pkgs.rustChannelOf {
    channel = "nightly";
    date = "2020-12-20";
  };

  rust = rustChannel.rust.override {
    extensions = [ "rust-src" ];
    targets = [ "riscv64gc-unknown-none-elf" ];
  };

  runQemu = pkgs.writers.writeBashBin "run-qemu" ''
    ${pkgs.qemu}/bin/qemu-system-riscv64 \
        -machine virt \
        -cpu rv64 \
        -smp 4 \
        -m 512M \
        -nographic \
        -serial mon:stdio \
        -bios none \
        -kernel "$@"
  '';

in pkgs.mkShell {
  name = "rust-shell";
  nativeBuildInputs = with pkgs;
    [ unstable.rust-analyzer rust llvm_11 qemu ] ++ [ runQemu ];
}
