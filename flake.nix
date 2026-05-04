{
  description = "AfterLivie development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ { flake-parts, treefmt-nix, git-hooks-nix, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [ treefmt-nix.flakeModule git-hooks-nix.flakeModule ];

      perSystem = { system, config, lib, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "llvm-tools-preview" ];
          };
          doctor = pkgs.writeShellApplication {
            name = "afterlivie-doctor";
            runtimeInputs = [
              rustToolchain
              pkgs.ffmpeg
            ];
            text = ''
              set -euo pipefail
              missing=0
              check() {
                if command -v "$1" >/dev/null 2>&1; then
                  printf "ok: %s -> %s\n" "$1" "$(command -v "$1")"
                else
                  printf "missing: %s\n" "$1"
                  missing=1
                fi
              }
              check rustc
              check cargo
              check ffmpeg
              check ffprobe
              check swift
              check xcodebuild
              if [ "$missing" -ne 0 ]; then
                exit 1
              fi
            '';
          };
        in
        {
          devShells.default = pkgs.mkShell {
            name = "afterlivie-dev";
            packages = with pkgs; [
              zsh
              direnv
              nix-direnv
              just
              treefmt
              editorconfig-checker
              typos
              shellcheck
              shfmt
              statix
              deadnix
              nixpkgs-fmt
              taplo
              actionlint
              pkg-config
              cmake
              ninja
              sqlite
              ffmpeg
              rustToolchain
              rust-analyzer
              cargo-nextest
              cargo-deny
              cargo-insta
              cargo-audit
              cargo-expand
              sccache
            ];
            shellHook = ''
              ${config.pre-commit.installationScript}
              export AFTERLIVIE_FFMPEG="${pkgs.ffmpeg}/bin/ffmpeg"
              export AFTERLIVIE_FFPROBE="${pkgs.ffmpeg}/bin/ffprobe"
              echo "afterlivie-dev: $(rustc -vV | head -n1), $(cargo -V)"
            '';
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              shfmt.enable = true;
              taplo.enable = true;
            };
          };

          pre-commit.settings.hooks = {
            actionlint.enable = true;
            cargo-check.enable = true;
            clippy.enable = true;
            deadnix.enable = true;
            editorconfig-checker.enable = true;
            nixpkgs-fmt.enable = true;
            shellcheck.enable = true;
            shfmt.enable = true;
            statix.enable = true;
            taplo.enable = true;
            typos.enable = true;
          };

          apps = {
            doctor = {
              type = "app";
              program = "${doctor}/bin/afterlivie-doctor";
            };
            format = {
              type = "app";
              program = "${config.treefmt.build.wrapper}/bin/treefmt";
            };
            test = {
              type = "app";
              program = "${pkgs.writeShellScript "afterlivie-test" ''
                set -euo pipefail
                cargo nextest run "$@"
              ''}";
            };
          };

          formatter = config.treefmt.build.wrapper;
        };
    };
}
