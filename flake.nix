{
  description = "A robust Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    
    # Add the community standard rust-overlay
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Import the rust-overlay
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system;
          inherit overlays;
        };

        # Use `rust.override` to add the `rust-src` extension
        rustToolchain = pkgs.rust-bin.stable.latest.rust.override {
          extensions = [ "rust-src" ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Add the complete toolchain
            rustToolchain

            # Add other tools, such as rust-analyzer
            pkgs.rust-analyzer
          ];
        };
      });
}
