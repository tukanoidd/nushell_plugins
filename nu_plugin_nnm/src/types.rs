use std::net::Ipv4Addr;

use nu_protocol::{LabeledError, PipelineData, Value};
use rusty_network_manager::NetworkManagerProxy;

use crate::{
    util::{ToLabeledError, ToLabeledResult, ToValue},
    value,
};

pub trait NMConnection<'a> {
    async fn new_ext(zbus_connection: &'a zbus::Connection) -> Result<Self, LabeledError>
    where
        Self: Sized;

    async fn status(
        self,
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
        self,
        zbus_connection: &zbus::Connection,
    ) -> Result<PipelineData, LabeledError> {
        Ok(PipelineData::Value(
            self.to_value(zbus_connection).await,
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
    NMCapability,
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
