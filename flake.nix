{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    gitignore.url = "github:hercules-ci/gitignore.nix";
    gitignore.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, gitignore }:
    let
      inherit (nixpkgs.lib) genAttrs getExe;

      forAllSystems = genAttrs [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      forAllPkgs = function: forAllSystems (system: function allPkgs.${system});

      mkApp = (program: { type = "app"; inherit program; });

      allPkgs = forAllSystems (system: (import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      }));
    in
    {
      formatter = forAllPkgs (pkgs: pkgs.nixpkgs-fmt);

      packages = forAllPkgs (pkgs: rec {
        default = wakatime-ls;
        wakatime-ls = pkgs.callPackage ./package.nix { inherit gitignore; };
      });
      apps = forAllSystems (system: rec {
        default = wakatime-ls;
        wakatime-ls = mkApp (getExe self.packages.${system}.wakatime-ls);
      });

      devShells = forAllPkgs (pkgs:
        let
          inherit (pkgs) lib fetchFromGitHub;
        
          file-rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rust-toolchain = file-rust-toolchain.override { extensions = [ "rust-analyzer" ]; };

          cargo-dist = pkgs.cargo-dist.overrideAttrs (final-attrs: old-attrs: {
            version = "0.28.5";
            src = fetchFromGitHub {
              owner = "astral-sh";
              repo = "cargo-dist";
              rev = "v${final-attrs.version}";
              hash = "sha256-SUMonuiX1xh1Fz77hf+v1I9nDIl9Am5B7Upv2zPcVJg=";
            };

            cargoHash = "sha256-cc/gCm9f86byXGVztMIbwoP2a8bmXk1r7ODhWFGa6IE=";
            cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
              inherit (final-attrs) pname src version;
              hash = final-attrs.cargoHash;
            };
          });
        in
        {
          default = pkgs.mkShell rec {
            nativeBuildInputs = with pkgs; [
              pkg-config
              rust-toolchain
              act

              cargo-dist

              wakatime-cli
            ];

            buildInputs = [ ];

            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
            LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

            RUST_LOG = "info";
          };
        });
    };
}
