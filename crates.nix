{...}: {
  perSystem = {pkgs, ...}: {
    nci.projects."nushell_plugins" = {
      path = ./.;
      export = true;
    };
    nci.crates = {
      "nu_plugin_graph" = {};
    };
  };
}
