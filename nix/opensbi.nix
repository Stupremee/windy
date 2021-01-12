# Nix Derivation for building OpenSBI.
# Output directory has following structure:
#
# platform/fw_{dynamic,jump,payload}.elf

{ platform ? "generic", pkgs }:
let
  inherit (pkgs) stdenv fetchFromGitHub;

  version = "0d49c3b";
in stdenv.mkDerivation rec {
  name = "opensbi";
  inherit version;

  src = fetchFromGitHub {
    owner = "riscv";
    repo = name;
    rev = "${version}";
    sha256 = "sha256-uqljriqyM2ydl6RCJvies+QphLmK7ytF1JJz4U8FGBQ=";
  };

  PLATFORM = platform;
  installPhase = ''
    mkdir -p $out/platform/
    mv ./build/platform/${platform}/firmware/{*.elf,*.bin} $out/platform
  '';
}
