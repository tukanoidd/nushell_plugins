mod plugin;
mod types;

use nu_plugin::{MsgPackSerializer, serve_plugin};
use plugin::NuGraphsPlugin;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    serve_plugin(&NuGraphsPlugin, MsgPackSerializer);
}
