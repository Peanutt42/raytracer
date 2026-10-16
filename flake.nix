{
  description = "raytracer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    let
      inherit (nixpkgs) lib;

      cargoToml = lib.importTOML ./Cargo.toml;
      pname = cargoToml.package.name;
      inherit (cargoToml.package) version;

      # `gpu = true` marks the ones that need the Vulkan/windowing libraries.
      binaries = {
        raytracer_gpu.gpu = true;
        raytracer_headless = { };
        raytracer_runtime.gpu = true;
        raytracer_terminal = { };
      };
      defaultBinary = "raytracer_gpu";
      gpuBinaries = lib.attrNames (
        lib.filterAttrs (
          _:
          {
            gpu ? false,
          }:
          gpu
        ) binaries
      );

      rustBinFor = pkgs: (pkgs.extend rust-overlay.overlays.default).rust-bin;
      toolchainFor = pkgs: (rustBinFor pkgs).selectLatestNightlyWith (toolchain: toolchain.minimal);
      devToolchainFor =
        pkgs:
        (rustBinFor pkgs).selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          }
        );
      # only needed by binaries with gpu = true
      runtimeLibsFor =
        pkgs: with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader

          # X11 fallback
          libx11
          libxcursor
          libxrandr
          libxi
        ];
      depsFor = pkgs: {
        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs = runtimeLibsFor pkgs;
      };
    in
    {
      overlays.default =
        final: _prev:
        let
          toolchain = toolchainFor final;
          rustPlatform = final.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };

          mkPackage =
            {
              pname,
              mainProgram,
              cargoFlags,
              deps ? {
                nativeBuildInputs = [ ];
                buildInputs = [ ];
              },
              patchBinaries ? [ ],
              doCheck ? true,
            }:
            rustPlatform.buildRustPackage (
              {
                inherit pname version doCheck;

                src = lib.fileset.toSource {
                  root = ./.;
                  fileset = lib.fileset.unions [
                    ./Cargo.toml
                    ./Cargo.lock
                    ./src
                    ./raytracer_gpu
                    ./raytracer_headless
                    ./raytracer_runtime
                    ./raytracer_terminal
                  ];
                };
                cargoLock.lockFile = ./Cargo.lock;
                cargoBuildFlags = cargoFlags;
                cargoTestFlags = cargoFlags;
                strictDeps = true;

                postFixup = lib.optionalString final.stdenv.hostPlatform.isLinux (
                  lib.concatMapStringsSep "\n" (
                    bin: "patchelf --add-rpath ${lib.makeLibraryPath (runtimeLibsFor final)} $out/bin/${bin}"
                  ) patchBinaries
                );

                meta = {
                  description = cargoToml.package.description or pname;
                  license = lib.licenses.mit;
                  inherit mainProgram;
                };
              }
              // deps
            );

          gpuArgsFor = bins: {
            deps = depsFor final;
            patchBinaries = bins;
          };
        in
        {
          ${pname} = mkPackage (
            {
              inherit pname;
              mainProgram = defaultBinary;
              cargoFlags = [ "--workspace" ];
            }
            // gpuArgsFor gpuBinaries
          );
        }
        # one package per binary
        // lib.mapAttrs (
          name:
          {
            gpu ? false,
          }:
          mkPackage (
            {
              pname = name;
              mainProgram = name;
              cargoFlags = [
                "-p"
                name
              ];
              doCheck = false; # covered by the workspace package (${pname})
            }
            // lib.optionalAttrs gpu (gpuArgsFor [ name ])
          )
        ) binaries;
    }
    // flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      in
      {
        packages = {
          default = pkgs.${defaultBinary};
          ${pname} = pkgs.${pname};
        }
        // lib.genAttrs (lib.attrNames binaries) (name: pkgs.${name});

        apps = {
          default = flake-utils.lib.mkApp { drv = pkgs.${defaultBinary}; };
        }
        // lib.mapAttrs (name: _: flake-utils.lib.mkApp { drv = pkgs.${name}; }) binaries;

        devShells.default = pkgs.mkShell {
          inherit (depsFor pkgs) nativeBuildInputs buildInputs;
          packages = [ (devToolchainFor pkgs) ];
          env.RUST_BACKTRACE = "1";
          shellHook = ''
            export LD_LIBRARY_PATH="${lib.makeLibraryPath (runtimeLibsFor pkgs)}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          '';
        };

        formatter = pkgs.nixfmt-tree;
      }
    );
}
