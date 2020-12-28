# Nix Derivation for building OpenSBI.
# Output directory has following structure:
#
# platform/fw_{dynamic,jump,payload}.elf

{ platform ? "generic", pkgs }:
let
  inherit (pkgs) stdenv fetchFromGitHub;
  version = "0.8";
in stdenv.mkDerivation rec {
  name = "opensbi";
  inherit version;

  src = fetchFromGitHub {
    owner = "riscv";
    repo = name;
    rev = "v${version}";
    sha256 = "sha256-C6V62FOZ6Gm1Ci6pcGrzqBomG6GVh+FfPfxyg80CP/k=";
  };

  PLATFORM = platform;
  installPhase = ''
    mkdir -p $out/platform/
    mv ./build/platform/${platform}/firmware/*.elf $out/platform
  '';
}
