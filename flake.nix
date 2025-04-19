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
        pkgs,
        ...
      }: let
        outputs = config.nci.outputs;

        external_plugin = {
          short_name,
          config ? {},
        }: {
          inherit short_name;
          inherit config;
        };
        external_plugins =
          builtins.map (plugin: let
            name = "nu_plugin_${plugin.short_name}";
          in rec {
            inherit name;

            short_name = plugin.short_name;
            config = plugin.config;
            path = inputs.${name};
          }) [
            (external_plugin {short_name = "clipboard";})
            (external_plugin {
              short_name = "audio_hook";
              config = let
                mkDrvConfig = {
                  nativeBuildInputs = with pkgs; [pkg-config];
                  buildInputs = with pkgs; [alsa-lib];
                };
              in {
                profiles = {
                  release.features = ["default" "all-decoders"];
                };
                drvConfig.mkDerivation = mkDrvConfig;
                depsDrvConfig.mkDerivation = mkDrvConfig;
              };
            })
            (external_plugin {short_name = "desktop_notifications";})
            (external_plugin {short_name = "emoji";})
            (external_plugin {short_name = "strutils";})
            (external_plugin {short_name = "file";})
            (external_plugin {short_name = "semver";})
            (external_plugin {short_name = "vec";})
            (external_plugin {short_name = "sled";})
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
              value = plugin.config;
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
          nixpkgs_plugin_packages = let
            ignored = ["regex"];
            not_ignored = name: !(builtins.elem name ignored);
          in
            lib.filterAttrs (n: v: (not_ignored n) && (lib.isDerivation v)) pkgs.nushellPlugins;
          other_plugin_packages = lib.mergeAttrs external_packages nixpkgs_plugin_packages;
        in (lib.mergeAttrs
          {
            graph = outputs."nu_plugin_graph".packages.release;
          }
          other_plugin_packages);
      };
    };
}
