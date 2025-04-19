use nu_plugin::PluginCommand;
use nu_protocol::{LabeledError, Signature};

use crate::run_with_nnm;

use super::NuNNMPlugin;

pub struct Version;

impl PluginCommand for Version {
    type Plugin = NuNNMPlugin;

    fn name(&self) -> &str {
        "nnm version"
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
    }

    fn description(&self) -> &str {
        "Version of NetworkManager"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &nu_plugin::EvaluatedCall,
        _input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, LabeledError> {
        run_with_nnm!(|zbus, nm| { nm.version().await })
    }
}

pub struct Status;

impl PluginCommand for Status {
    type Plugin = NuNNMPlugin;

    fn name(&self) -> &str {
        "nnm status"
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
    }

    fn description(&self) -> &str {
        "Network Status"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &nu_plugin::EvaluatedCall,
        _input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::LabeledError> {
        run_with_nnm!(|zbus, nm| { nm.status(&zbus).await })
    }
}
