mod plugin;
mod traits;
mod util;

use nu_plugin::{MsgPackSerializer, serve_plugin};
use plugin::NuNNMPlugin;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    serve_plugin(&NuNNMPlugin, MsgPackSerializer);
}
