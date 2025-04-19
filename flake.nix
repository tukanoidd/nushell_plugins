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

    nu_plugin_clipboard = {
      url = "github:FMotalleb/nu_plugin_clipboard";
      flake = false;
    };
    nu_plugin_audio_hook = {
      url = "github:FMotalleb/nu_plugin_audio_hook";
      flake = false;
    };
    nu_plugin_desktop_notifications = {
      url = "github:FMotalleb/nu_plugin_desktop_notifications";
      flake = false;
    };
    nu_plugin_emoji = {
      url = "github:fdncred/nu_plugin_emoji";
      flake = false;
    };
    nu_plugin_strutils = {
      url = "github:fdncred/nu_plugin_strutils";
      flake = false;
    };
    nu_plugin_file = {
      url = "github:fdncred/nu_plugin_file";
      flake = false;
    };
    nu_plugin_semver = {
      url = "github:abusch/nu_plugin_semver";
      flake = false;
    };
    nu_plugin_vec = {
      url = "github:PhotonBursted/nu_plugin_vec";
      flake = false;
    };
    nu_plugin_sled = {
      url = "github:mrxiaozhuox/nu_plugin_sled";
      flake = false;
    };
  };

  outputs = inputs @ {
    parts,
    nci,
    ...
  }:
    parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      imports = [nci.flakeModule];

      perSystem = {
        config,
        lib,
        ...
      }: let
        outputs = config.nci.outputs;

        external_plugins =
          builtins.map (short_name: let
            name = "nu_plugin_${short_name}";
          in rec {
            inherit name;
            inherit short_name;

            path = inputs.${name};
          }) [
            "clipboard"
            "audio_hook"
            "desktop_notifications"
            "emoji"
            "strutils"
            "file"
            "semver"
            "vec"
            "sled"
          ];
      in {
        nci = let
          external_projects = builtins.listToAttrs (builtins.map (plugin: {
              name = plugin.name;
              value = {
                path = plugin.path;
                export = true;
              };
            })
            external_plugins);
          external_crates = builtins.listToAttrs (builtins.map (plugin: {
              name = plugin.name;
              value = {};
            })
            external_plugins);
        in {
          projects =
            lib.mergeAttrs {
              "nushell_plugins" = {
                path = ./.;
                export = true;
              };
            }
            external_projects;

          crates =
            lib.mergeAttrs
            {
              "nu_plugin_graph" = {};
            }
            external_crates;
        };

        devShells.default = outputs."nushell_plugins".devShell;
        packages = let
          external_packages = builtins.listToAttrs (builtins.map (plugin: {
              name = plugin.short_name;
              value = outputs.${plugin.name}.packages.release;
            })
            external_plugins);
        in (lib.mergeAttrs
          {
            graph = outputs."nu_plugin_graph".packages.release;
          }
          external_packages);
      };
    };
}
