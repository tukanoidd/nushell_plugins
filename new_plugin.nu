#!/usr/bin/env nu

def main [name: string] {
  let full_name = $"nu_plugin_($name)"

  cargo init --lib $full_name
  git add $full_name
  cd $full_name

  # Overwrite Cargo.toml
  (
    open Cargo.toml
    | update package {|pkg|
      $pkg.package | insert build "build.rs"
    } | update dependencies {|deps|
      $deps.dependencies | merge {
        nu-plugin: {workpace: true},
        nu-protocol: {workspace: true}
      }
    } | upsert build-dependencies {
      built: {workspace: true}
    }
  ) | to toml | save -f Cargo.toml

  def spawn_file [path: path, lines: list<string>]: nothing -> nothing {
    $lines | str join "\n" | save -f $path
  }

  # Spawn build.rs
  (spawn_file build.rs     [
    "fn main() {",
    "\tbuilt::write_built_file().expect(\"Failed to acquire build-time information\");",
    "}"
  ])

  (spawn_file src/main.rs [
    "mod plugin;"
    "mod types;"
    ""
    "use nu_plugin::{serve_plugin, MsgPackSerializer};"
    "use plugin::NuRenamePlugin;"
    ""
    "pub mod built_info {"
      "\tinclude!(concat!(env!("OUT_DIR"), "/built.rs"));"
    "}"
    ""
    "fn main() {"
    "\tserve_plugin(&NuRenamePlugin, MsgPackSerializer);"
    "}"
  ])

  (spawn_file src/plugin.rs [
    "mod commands;"
    ""
    "use commands::Draw;"
    "use nu_plugin::Plugin;"
    ""
    "use crate::built_info;"

    "pub struct NuGraphsPlugin;"

    "impl Plugin for NuGraphsPlugin {"
    "\tfn version(&self) -> String {"
    "\t\tbuilt_info::PKG_VERSION.into()"
    "\t}"
    "",
    "\tfn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {"
    "\t\tvec![]"
    "\t}"
    "}"
  ])

  if not ("src/plugin/" | path exists) {
    mkdir src/plugin/
  }

  touch src/plugin/commands.rs

  cd ..
}
