{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    parts,
    nci,
    ...
  }:
    parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      imports = [nci.flakeModule ./crates.nix];
      perSystem = {config, ...}: let
        outputs = config.nci.outputs;
      in {
        devShells.default = outputs."nushell_plugins".devShell;
        packages = {
          graph = outputs."nu_plugin_graph".packages.release;
        };
      };
    };
}
