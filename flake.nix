{
  inputs = {
    naersk.url = "github:nmattia/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, naersk, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        
        allBuildInputs = with pkgs; [
          # Rust tools
          cargo
          rust-analyzer
          rustc
          rustfmt
          
          # Your original dev tools
          cargo-insta
          pre-commit
          rustPackages.clippy
          tokei
        
          # --- SDL2 Dependencies (for emulator) ---
          SDL2
          SDL2_image
          SDL2_ttf
          SDL2_mixer
          
          # --- eframe Dependencies (for menu) ---
          pkg-config
          wayland
          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          xorg.libxcb
          vulkan-loader
          mesa
          libglvnd
          libGL
          glibcLocales
          xorg.xkeyboardconfig

          # --- THIS IS THE FIX ---
          # Add the program that native-dialog uses
          zenity 
        ];
        
        libPath = with pkgs; lib.makeLibraryPath allBuildInputs;
      in
      {
        # ... (rest of the file is unchanged) ...
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          doCheck = true;
          pname = "sixty-two";
          nativeBuildInputs = [ pkgs.makeWrapper ];
          buildInputs = with pkgs; [
            xorg.libxcb
          ];
          postInstall = ''
            wrapProgram "$out/bin/sixty-two" --prefix LD_LIBRARY_PATH : "${libPath}"
          '';
        };

        defaultApp = utils.lib.mkApp {
          drv = self.defaultPackage."${system}";
        };

        devShell = with pkgs; mkShell {
          buildInputs = allBuildInputs;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = libPath;

          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPath}";
            export LOCALE_ARCHIVE="${glibcLocales}/lib/locale/locale-archive"
            export LANG="en_IN.UTF-8"
            export XKB_CONFIG_ROOT="${xorg.xkeyboardconfig}/share/X11/xkb"
          '';
        };
      });
}
