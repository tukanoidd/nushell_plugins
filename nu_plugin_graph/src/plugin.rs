mod commands;

use commands::Draw;
use nu_plugin::Plugin;

use crate::built_info;

pub struct NuGraphsPlugin;

impl Plugin for NuGraphsPlugin {
    fn version(&self) -> String {
        built_info::PKG_VERSION.into()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(Draw)]
    }
}
