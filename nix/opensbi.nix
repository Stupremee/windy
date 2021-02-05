# Nix Derivation for building OpenSBI.
# Output directory has following structure:
#
# platform/fw_{dynamic,jump,payload}.elf

{ platform ? "generic", pkgs }:
let
  inherit (pkgs) stdenv fetchFromGitHub;

  version = "master";
in stdenv.mkDerivation rec {
  name = "opensbi";
  inherit version;

  src = fetchFromGitHub {
    owner = "riscv";
    repo = name;
    rev = "234ed8e427f4d92903123199f6590d144e0d9351";
    sha256 = "sha256-uqljriqyM2ydl6RCJvies+QphLmK7ytF1JJz4U8FGBQ=";
  };

  PLATFORM = platform;
  installPhase = ''
    mkdir -p $out/platform/
    mv ./build/platform/${platform}/firmware/{*.elf,*.bin} $out/platform
  '';
}
