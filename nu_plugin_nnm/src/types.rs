use std::net::Ipv4Addr;

use futures::future::join_all;
use nu_protocol::{LabeledError, PipelineData, Span, Value};
use rusty_network_manager::{ActiveProxy, NetworkManagerProxy};

use crate::{
    util::{ToLabeledError, ToLabeledResult, ToValOrNothing, ToValue},
    value,
};

pub trait NMConnection<'a> {
    async fn new_ext(zbus_connection: &'a zbus::Connection) -> Result<Self, LabeledError>
    where
        Self: Sized;

    async fn status(
        &self,
        zbus_connection: &'a zbus::Connection,
    ) -> Result<PipelineData, LabeledError>;
}

impl<'a> NMConnection<'a> for NetworkManagerProxy<'a> {
    async fn new_ext(zbus_connection: &'a zbus::Connection) -> Result<Self, LabeledError> {
        let proxy = NetworkManagerProxy::new(zbus_connection)
            .await
            .to_labeled()?;

        Ok(proxy)
    }

    async fn status(
        &self,
        zbus_connection: &zbus::Connection,
    ) -> Result<PipelineData, LabeledError> {
        // TODO: activating_connection

        let active_connection_paths = self.active_connections().await.to_labeled()?;
        let active_connections: Vec<_> = join_all(active_connection_paths.into_iter().map(
            |connection_path| async {
                let active_proxy =
                    ActiveProxy::new_from_path(connection_path.clone(), zbus_connection)
                        .await
                        .ok()?;

                Some(
                    (connection_path, active_proxy)
                        .to_value(zbus_connection)
                        .await,
                )
            },
        ))
        .await
        .into_iter()
        .flatten()
        .collect();

        // TODO: all_devices
        // TODO: capabilities
        // TODO: checkpoints
        // TODO: connectivity
        // TODO: connectivity_check_available
        // TODO: connectivity_check_enabled
        // TODO: devices
        // TODO: global_dns_configuration
        // TODO: metered
        // TODO: networking_enabled
        // TODO: primary_connection
        // TODO: primary_connection_type
        // TODO: radio_flags
        // TODO: startup

        let version = self.version().await.to_val_or_nothing_with(Value::string);

        // TODO: version_info
        // TODO: wimax_enabled
        // TODO: wimax_hardware_enabled
        // TODO: wireless_enabled
        // TODO: wireless_hardware_enabled
        // TODO: wwan_enabled
        // TODO: wwan_hardware_enabled

        Ok(PipelineData::Value(
            value!({
                // TODO: activating_connection
                "active_connections": Value::list(
                    active_connections,
                    Span::unknown()
                ),
                // TODO: all_devices
                // TODO: capabilities
                // TODO: checkpoints
                // TODO: connectivity
                // TODO: connectivity_check_available
                // TODO: connectivity_check_enabled
                // TODO: devices
                // TODO: global_dns_configuration
                // TODO: metered
                // TODO: networking_enabled
                // TODO: primary_connection
                // TODO: primary_connection_type
                // TODO: radio_flags
                // TODO: startup
                "version": version,
            }),
            None,
        ))
    }
}

pub trait ValFromU32 {
    fn from_u32_to_str_val(val: u32) -> Value;
}

pub trait NMFlags: TryFrom<u32, Error: ToLabeledError> + std::fmt::Debug {}

macro_rules! impl_nm_flags {
    ($($name:ident),+) => {$(
        impl NMFlags for rusty_network_manager::dbus_interface_types::$name {}
    )+};
}

impl_nm_flags![
    NMActivationStateFlags,
    NMActiveConnectionState,
    NMConnectivityState,
    NMDeviceCapabilities,
    NMDeviceInterfaceFlags,
    NMDeviceState,
    NMDeviceStateReason,
    NMMetered
];

impl<F> ValFromU32 for F
where
    F: NMFlags,
{
    fn from_u32_to_str_val(val: u32) -> Value {
        Self::try_from(val)
            .map(|s| value!(string format!("{s:?}")))
            .to_labeled()
            .unwrap_or_else(|_| value!(int val as i64))
    }
}

impl ValFromU32 for Ipv4Addr {
    fn from_u32_to_str_val(val: u32) -> Value {
        value!(string Ipv4Addr::from_bits(val).to_string())
    }
}
