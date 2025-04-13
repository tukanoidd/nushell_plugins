mod plugin;
mod types;

use nu_plugin::{serve_plugin, MsgPackSerializer};
use plugin::NuGraphsPlugin;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    serve_plugin(&NuGraphsPlugin, MsgPackSerializer);
}
