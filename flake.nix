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
        
        # 1. Runtime Libraries (Needed to run the app)
        runtimeDeps = with pkgs; [
          SDL2
          SDL2_image
          SDL2_ttf
          SDL2_mixer
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
          zenity # Needed for native-dialog
        ];

        # 2. Build Tools (Needed to compile the app)
        buildDeps = with pkgs; [
          pkg-config
          makeWrapper
        ];

        # 3. Development Tools (ONLY for your shell, not the build)
        devDeps = with pkgs; [
          cargo
          rustc
          rust-analyzer
          rustfmt
          cargo-insta
          pre-commit
          rustPackages.clippy
          tokei
        ];

        libPath = with pkgs; lib.makeLibraryPath runtimeDeps;
      in
      {
        # The Package (Clean build, no dev tools)
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          doCheck = true;
          pname = "nesemu"; 
          
          nativeBuildInputs = buildDeps;
          buildInputs = runtimeDeps;

          postInstall = ''
            wrapProgram "$out/bin/nesemu" \
              --prefix LD_LIBRARY_PATH : "${libPath}" \
              --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.zenity ]}"
          '';
        };

        # The Shell (Includes everything: dev tools + build deps + libraries)
        devShell = with pkgs; mkShell {
          buildInputs = runtimeDeps ++ buildDeps ++ devDeps;
          
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
