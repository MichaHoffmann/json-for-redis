let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> {
    overlays = [ (import rustOverlay) ];
  };
in
pkgs.mkShell {
  name = "env";
  nativeBuildInputs = with pkgs; [
    rust-bin.nightly.latest.default
    llvmPackages_latest.llvm
    llvmPackages_latest.bintools
    clang
    redis
    pre-commit
  ];
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
  RUST_BACKTRACE = 1;
}

