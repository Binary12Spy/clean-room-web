{
  description = "clean-room-web — reproducible dev environment for the PoC host and WASM bundles";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Single pinned toolchain used for BOTH the native host and the
        # wasm32-unknown-unknown bundles. The bundles need the wasm target;
        # rust-src + rust-analyzer make the editor experience sane.
        rustToolchain = pkgs.rust-bin.stable."1.96.0".default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" "llvm-tools" ];
        };

        # Runtime libraries that winit + softbuffer dlopen at runtime on Linux.
        # These must be discoverable via LD_LIBRARY_PATH for the host window to open.
        runtimeLibs = with pkgs; [
          # X11 stack
          libx11
          libxcursor
          libxi
          libxrandr
          libxcb
          # Wayland stack (present so the binary runs under either session)
          wayland
          # keyboard handling (winit)
          libxkbcommon
          # GL (softbuffer can fall back through it on some setups)
          libGL
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          name = "clean-room-web";

          packages = [
            rustToolchain
          ] ++ (with pkgs; [
            pkg-config        # crates that probe system libs at build time
            wasm-tools        # inspect / validate produced .wasm (wasm-tools print, validate)
            binaryen          # wasm-opt, if we want to size-optimize bundles later
            cargo-watch       # convenience for the edit/build loop
          ]) ++ runtimeLibs;

          # dlopen'd libraries are not found via rpath; expose them explicitly.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;

          shellHook = ''
            echo "clean-room-web dev shell"
            echo "  rustc:   $(rustc --version)"
            echo "  targets: native + wasm32-unknown-unknown"
            echo "  build:   cd architecture/poc && cargo xtask build"
          '';
        };
      });
}
