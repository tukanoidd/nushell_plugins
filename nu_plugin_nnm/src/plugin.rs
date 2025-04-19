mod commands;

use nu_plugin::Plugin;

use crate::built_info;

pub struct NuNNMPlugin;

macro_rules! commands {
    ($($name:ident),+) => {
        vec![$(Box::new(commands::$name)),+]
    }
}

impl Plugin for NuNNMPlugin {
    fn version(&self) -> String {
        built_info::PKG_VERSION.into()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        commands![Version, Status]
    }
}
