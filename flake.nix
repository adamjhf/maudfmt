{
  description = "simple formatter for maud macros";

  inputs = {
    # NixPkgs
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Fenix: rust toolchain
    fenix.url = "github:nix-community/fenix/monthly";

    # Naersk: rust packager
    naersk.url = "github:nix-community/naersk";

    # Pre-commit hooks
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      naersk,
      pre-commit-hooks,
      ...
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      mkRustToolchain =
        system:
        with fenix.packages.${system};
        combine [
          stable.cargo
          stable.rustc
          stable.rustfmt
          stable.clippy
        ];
    in
    {
      apps = forAllSystems (system: rec {
        default = maudfmt;
        maudfmt = {
          type = "app";
          program = "${self.packages.${system}.maudfmt}/bin/maudfmt";
        };
      });
      packages = forAllSystems (
        system:
        let
          toolchain = mkRustToolchain system;
          naersk' = naersk.lib.${system}.override {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        rec {
          default = maudfmt;
          maudfmt = naersk'.buildPackage {
            src = ./.;
            doCheck = true; # run tests when compiling
          };
        }
      );

      devShells = forAllSystems (system: {
        default =
          let
            pkgs = import nixpkgs { inherit system; };
            toolchain = mkRustToolchain system;
          in
          pkgs.mkShell rec {
            packages =
              with pkgs;
              [
                # General
                just

                # Rust
                toolchain
                rust-analyzer
                bacon
              ]
              ++ self.checks.${system}.pre-commit-check.enabledPackages;

            inherit (self.checks.${system}.pre-commit-check) shellHook;

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
          };
      });

      checks = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          toolchain = mkRustToolchain system;
        in
        {
          pre-commit-check = pre-commit-hooks.lib.${pkgs.system}.run {
            src = ./.;
            settings = {
              rust.check.cargoDeps = pkgs.rustPlatform.importCargoLock {
                lockFile = ./Cargo.lock;
              };
            };
            hooks = {
              # General
              yamlfmt.enable = true;
              keep-sorted = {
                enable = true;
                name = "Keep sorted";
                types = [
                  "nix"
                  "rust"
                ];
                entry = "${pkgs.keep-sorted}/bin/keep-sorted";
              };
              readme-help = {
                enable = true;
                name = "Generate readme help";
                entry = "${pkgs.just}/bin/just update-readme-help";
                pass_filenames = false;
              };
              # Nix
              nixfmt-rfc-style.enable = true;
              flake-checker.enable = true;
              # Rust
              rustfmt = {
                enable = true;
                packageOverrides = {
                  cargo = toolchain;
                  rustfmt = toolchain;
                };
              };
              clippy = {
                enable = true;
                packageOverrides = {
                  cargo = toolchain;
                  clippy = toolchain;
                };
                settings = {
                  denyWarnings = true;
                  extraArgs = "--all-targets";
                };
              };
            };
          };
        }
      );
    };
}
