{
  description = "SIAR - Sovereign, Interoperable, Asynchronous, Resilient Communications System";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs) lib;

        # Rust toolchain matching rust-toolchain.toml (stable Rust)
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Linux-specific build requirements for Desktop & Media
        desktopLinuxNativeBuildInputs = with pkgs; [
          pkg-config
          wrapGAppsHook3
          gobject-introspection
        ];

        desktopLinuxBuildInputs = with pkgs; [
          webkitgtk_4_1
          gtk3
          glib
          cairo
          pango
          gdk-pixbuf
          libsoup_3
          dav1d
          libopus
          alsa-lib
          openssl
          dbus
          gsettings-desktop-schemas
        ];

        # Core headless / CLI Linux dependencies
        coreLinuxBuildInputs = with pkgs; [
          openssl
          dav1d
          libopus
          alsa-lib
          dbus
        ];

        # Darwin-specific build requirements
        darwinBuildInputs = with pkgs; [
          libopus
          dav1d
          openssl
          libiconv
        ] ++ lib.optionals pkgs.stdenv.isDarwin (
          if pkgs ? apple-sdk_11 then [
            pkgs.apple-sdk_11
          ] else if pkgs ? darwin && pkgs.darwin ? apple_sdk then (with pkgs.darwin.apple_sdk.frameworks; [
            Security
            CoreServices
            CoreAudio
            AudioToolbox
            AppKit
            WebKit
            Foundation
            CoreGraphics
          ]) else [ ]
        );

        # Common dependencies for all workspace cargo artifacts
        commonNativeBuildInputs = with pkgs; [
          pkg-config
        ];

        commonBuildInputs = with pkgs; [
          openssl
          dav1d
          libopus
        ]
        ++ lib.optionals pkgs.stdenv.isLinux desktopLinuxBuildInputs
        ++ lib.optionals pkgs.stdenv.isDarwin darwinBuildInputs;

        # Filter source to include cargo files, source code, and static assets
        assetFilter = path: _type:
          builtins.match ".*\\.(png|jpg|jpeg|svg|ico|css|sql|json|toml|md)$" path != null;

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type) || (assetFilter path type);
        };

        # Common derivation arguments
        commonArgs = {
          inherit src;
          pname = "siar-workspace";
          version = "0.1.0";
          strictDeps = true;
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        };

        # Pre-built Cargo dependency artifacts (shared cached layer)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Packages
        siar-cli = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "siar";
          cargoExtraArgs = "--package siar-cli --bin siar";
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = if pkgs.stdenv.isLinux then coreLinuxBuildInputs else commonBuildInputs;
          meta = with lib; {
            description = "SIAR CLI - Sovereign, Interoperable, Asynchronous, Resilient CLI client";
            homepage = "https://github.com/irshadali5/siar";
            license = licenses.agpl3Plus;
            mainProgram = "siar";
          };
        });

        siar-emergency-node = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "siar-emergency-node";
          cargoExtraArgs = "--package siar-emergency-node --bin siar-emergency-node";
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = if pkgs.stdenv.isLinux then coreLinuxBuildInputs else commonBuildInputs;
          meta = with lib; {
            description = "SIAR Emergency DTN Mesh Node daemon";
            homepage = "https://github.com/irshadali5/siar";
            license = licenses.agpl3Plus;
            mainProgram = "siar-emergency-node";
          };
        });

        siar-desktop = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "siar-desktop";
          cargoExtraArgs = "--package siar-desktop --bin siar-desktop";
          nativeBuildInputs = [ pkgs.pkg-config ] ++ lib.optionals pkgs.stdenv.isLinux desktopLinuxNativeBuildInputs;
          buildInputs = commonBuildInputs;
          meta = with lib; {
            description = "SIAR Desktop GUI client (Dioxus)";
            homepage = "https://github.com/irshadali5/siar";
            license = licenses.agpl3Plus;
            mainProgram = "siar-desktop";
          };
        });

        siar-all = pkgs.symlinkJoin {
          name = "siar-all";
          paths = [ siar-cli siar-emergency-node siar-desktop ];
        };

        # Check derivations
        checks = {
          inherit siar-cli siar-emergency-node;

          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace --all-targets";
          });

          fmt = craneLib.cargoFmt {
            inherit src;
          };

          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--workspace";
          });
        } // lib.optionalAttrs pkgs.stdenv.isLinux {
          inherit siar-desktop;
        };
      in
      {
        inherit checks;

        packages = {
          default = siar-cli;
          siar = siar-cli;
          siar-cli = siar-cli;
          siar-emergency-node = siar-emergency-node;
          siar-desktop = siar-desktop;
          all = siar-all;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = siar-cli;
            name = "siar";
          };
          siar = flake-utils.lib.mkApp {
            drv = siar-cli;
            name = "siar";
          };
          siar-emergency-node = flake-utils.lib.mkApp {
            drv = siar-emergency-node;
            name = "siar-emergency-node";
          };
          siar-desktop = flake-utils.lib.mkApp {
            drv = siar-desktop;
            name = "siar-desktop";
          };
        };

        devShells.default = craneLib.devShell {
          inherit checks;

          packages = with pkgs; [
            rustToolchain
            cargo-deny
            cargo-audit
            cargo-fuzz
            nixfmt-rfc-style
          ]
          ++ commonNativeBuildInputs
          ++ commonBuildInputs;

          shellHook = ''
            export RUST_BACKTRACE=1
            ${lib.optionalString pkgs.stdenv.isLinux ''
              export LD_LIBRARY_PATH="${lib.makeLibraryPath commonBuildInputs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
              export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}''${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
            ''}
            echo "================================================================="
            echo "  SIAR Hermetic Development Environment (Nix Flake)"
            echo "================================================================="
            echo "  Rust:         $(rustc --version 2>/dev/null || echo 'managed by toolchain')"
            echo "  Cargo:        $(cargo --version 2>/dev/null || echo 'managed by toolchain')"
            echo ""
            echo "  Build packages:"
            echo "    nix build .#siar-cli"
            echo "    nix build .#siar-desktop"
            echo "    nix build .#siar-emergency-node"
            echo "    nix build .#all"
            echo ""
            echo "  Run checks:"
            echo "    nix flake check"
            echo "================================================================="
          '';
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
