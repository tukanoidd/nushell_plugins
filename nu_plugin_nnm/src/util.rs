use std::net::Ipv4Addr;

use futures::future::join_all;
use nu_protocol::{LabeledError, Record, Span, Value};
use rusty_network_manager::{
    ActiveProxy, ConnectionProxy, DeviceProxy, NetworkManagerProxy,
    dbus_interface_types::{
        NMActivationStateFlags, NMActiveConnectionState, NMCapability, NMConnectivityState,
        NMDeviceCapabilities, NMDeviceInterfaceFlags, NMDeviceState, NMDeviceStateReason,
        NMMetered,
    },
};
use zbus::zvariant::OwnedObjectPath;

use crate::traits::ValFromU32;

#[macro_export]
macro_rules! run_with_nnm {
    (|$zbus:ident, $nm:ident| $block:block) => {{
        use $crate::{traits::*, util::*};

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .to_labeled()?;

        runtime.block_on(async move {
            let $zbus = zbus::Connection::system().await.to_labeled()?;
            let $nm: rusty_network_manager::NetworkManagerProxy =
                rusty_network_manager::NetworkManagerProxy::new_ext(&$zbus).await?;

            $block
        })
    }};
}

#[macro_export]
macro_rules! value {
    () => {
        nu_protocol::Value::nothing(nu_protocol::Span::unknown())
    };
    ({$($field:literal: $val:expr),* $(,)?}) => {
        {
            use $crate::util::ValueExt;

            nu_protocol::Value::record_str_iter([$(
                ($field, $val)
            ),*])
        }
    };

    ($name:ident $expr:expr) => {
        nu_protocol::Value::$name($expr, nu_protocol::Span::unknown())
    };
}

pub trait ValueExt {
    fn record_str_iter<'a>(iter: impl IntoIterator<Item = (&'a str, Value)>) -> Self;
}

impl ValueExt for Value {
    fn record_str_iter<'a>(iter: impl IntoIterator<Item = (&'a str, Value)>) -> Self {
        Value::record(Record::from_str_iter(iter), Span::unknown())
    }
}

pub trait RecordExt {
    fn from_str_iter<'a>(iter: impl IntoIterator<Item = (&'a str, Value)>) -> Self
    where
        Self: Sized;
}

impl RecordExt for Record {
    fn from_str_iter<'a>(iter: impl IntoIterator<Item = (&'a str, Value)>) -> Self
    where
        Self: Sized,
    {
        Record::from_iter(iter.into_iter().map(|(key, val)| (key.into(), val)))
    }
}

pub trait ToValue {
    async fn to_value(self, _zbus_connection: &zbus::Connection) -> Value;
}

impl<'a> ToValue for NetworkManagerProxy<'a> {
    async fn to_value(self, zbus_connection: &zbus::Connection) -> Value {
        // TODO: activating_connection

        let Some(active_connection_paths) = self.active_connections().await.ok() else {
            return value!();
        };
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

        let Some(all_device_paths) = self.all_devices().await.ok() else {
            return value!();
        };
        let all_devices = {
            value!(list join_all(all_device_paths.into_iter().map(|device_path| async {
                let device_proxy = DeviceProxy::new_from_path(device_path, zbus_connection).await.ok()?;
                Some(device_proxy.to_value(zbus_connection).await)
            })).await.into_iter().flatten().collect())
        };

        let Some(capabilities_paths) = self.capabilities().await.ok() else {
            return value!();
        };
        let capabilities = value!(list capabilities_paths.into_iter().map(NMCapability::from_u32_to_str_val).collect());

        // TODO: checkpoints

        let connectivity = self
            .connectivity()
            .await
            .map(NMConnectivityState::from_u32_to_str_val)
            .unwrap_or_nothing();
        let connectivity_check_available = self
            .connectivity_check_available()
            .await
            .to_val_or_nothing_with(Value::bool);
        let connectivity_check_enabled = self
            .connectivity_check_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);

        let Some(device_paths) = self.devices().await.ok() else {
            return value!();
        };
        let devices = {
            value!(list join_all(device_paths.into_iter().map(|device_path| async {
                let device_proxy = DeviceProxy::new_from_path(device_path, zbus_connection).await.ok()?;
                Some(device_proxy.to_value(zbus_connection).await)
            })).await.into_iter().flatten().collect())
        };

        // TODO: global_dns_configuration

        let metered = self
            .metered()
            .await
            .map(NMMetered::from_u32_to_str_val)
            .unwrap_or_nothing();
        let networking_enabled = self
            .networking_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);

        // TODO: primary_connection

        let primary_connection_type = self
            .primary_connection_type()
            .await
            .to_val_or_nothing_with(Value::string);

        // TODO: radio_flags

        let startup = self.startup().await.to_val_or_nothing_with(Value::bool);
        let version = self.version().await.to_val_or_nothing_with(Value::string);

        // TODO: version_info

        let wimax_enabled = self
            .wimax_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);
        let wimax_hardware_enabled = self
            .wimax_hardware_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);

        let wireless_enabled = self
            .wireless_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);
        let wireless_hardware_enabled = self
            .wireless_hardware_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);

        let wwan_enabled = self
            .wwan_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);
        let wwan_hardware_enabled = self
            .wwan_hardware_enabled()
            .await
            .to_val_or_nothing_with(Value::bool);

        value!({
            // TODO: activating_connection

            "active_connections": Value::list(
                active_connections,
                Span::unknown()
            ),
            "all_devices": all_devices,
            "capabilities": capabilities,

            // TODO: checkpoints

            "connectivity": value!({
                "state": connectivity,
                "check": value!({
                    "enabled": connectivity_check_enabled,
                    "available": connectivity_check_available,
                }),
            }),
            "devices": devices,

            // TODO: global_dns_configuration

            "metered": metered,
            "networking_enabled": networking_enabled,
            "primary_connetion": value!({
                // TODO: primary_connection
                "info": value!({}),
                "type": primary_connection_type
            }),

            // TODO: radio_flags

            "startup": startup,
            "version": version,

            // TODO: version_info

            "wimax": value!({
                "enabled": wimax_enabled,
                "hardware_enabled": wimax_hardware_enabled
            }),
            "wireless": value!({
                "enabled": wireless_enabled,
                "hardware_enabled": wireless_hardware_enabled
            }),
            "wwan": value!({
                "enabled": wwan_enabled,
                "hardware_enabled": wwan_hardware_enabled
            }),
        })
    }
}

impl<'a> ToValue for (OwnedObjectPath, ActiveProxy<'a>) {
    async fn to_value(self, zbus_connection: &zbus::Connection) -> Value {
        let (path, proxy) = self;

        let default = proxy.default().await.to_val_or_nothing_with(Value::bool);
        let default6 = proxy.default6().await.to_val_or_nothing_with(Value::bool);

        let devices = match proxy.devices().await.ok() {
            Some(paths) => {
                let devices = join_all(paths.into_iter().map(|path| async move {
                    match DeviceProxy::new_from_path(path, zbus_connection).await.ok() {
                        Some(proxy) => Some(proxy.to_value(zbus_connection).await),
                        None => None,
                    }
                }))
                .await
                .into_iter()
                .flatten()
                .collect();

                Value::list(devices, Span::unknown())
            }
            None => value!(),
        };

        // TODO: dhcp4_config
        // TODO: dhcp6_config

        let id = proxy.id().await.to_val_or_nothing_with(Value::string);

        // TODO: ip4_config
        // TODO: ip6_config
        // TODO: master
        // TODO: specific_object

        let state = proxy
            .state()
            .await
            .to_labeled()
            .map(NMActiveConnectionState::from_u32_to_str_val)
            .unwrap_or_nothing();
        let state_flags = proxy
            .state_flags()
            .await
            .map(NMActivationStateFlags::from_u32_to_str_val)
            .unwrap_or_nothing();

        let type_ = proxy.type_().await.to_val_or_nothing_with(Value::string);
        let uuid = proxy.uuid().await.to_val_or_nothing_with(Value::string);

        let vpn = match proxy.vpn().await.ok().unwrap_or_default() {
            true => {
                match ConnectionProxy::new_from_path(path.clone(), zbus_connection)
                    .await
                    .ok()
                {
                    Some(connection_proxy) => connection_proxy.to_value(zbus_connection).await,
                    None => value!(),
                }
            }
            false => value!(),
        };

        value!({
            "default": default,
            "default6": default6,
            "devices": devices,

            // TODO: dhcp4_config
            // TODO: dhcp6_config

            "id": id,

            // TODO: ip4_config
            // TODO: ip6_config
            // TODO: master
            // TODO: specific_object

            "state": state,
            "state_flags": state_flags,
            "type": type_,
            "uuid": uuid,
            "vpn": vpn,
        })
    }
}

impl<'a> ToValue for ConnectionProxy<'a> {
    async fn to_value(self, _zbus_connection: &zbus::Connection) -> Value {
        let banner = self.banner().await.to_val_or_nothing_with(Value::string);

        // TODO: proper representation
        let vpn_state = self
            .vpn_state()
            .await
            .to_labeled()
            .map(|val| value!(int val as i64))
            .unwrap_or_nothing();

        value!({
            "banner": banner,
            "vpn_state": vpn_state
        })
    }
}

impl<'a> ToValue for DeviceProxy<'a> {
    async fn to_value(self, _zbus_connection: &zbus::Connection) -> Value {
        let capabilities = self
            .capabilities()
            .await
            .to_labeled()
            .map(NMDeviceCapabilities::from_u32_to_str_val)
            .unwrap_or_nothing();

        // TODO: device_type
        // TODO: dhcp4_config
        // TODO: dhcp6_config

        let driver = self.driver().await.to_val_or_nothing_with(Value::string);
        let driver_version = self
            .driver_version()
            .await
            .to_val_or_nothing_with(Value::string);
        let firmware_missing = self
            .firmware_missing()
            .await
            .to_val_or_nothing_with(Value::bool);
        let firmware_version = self
            .firmware_version()
            .await
            .to_val_or_nothing_with(Value::string);
        let hw_address = self
            .hw_address()
            .await
            .to_val_or_nothing_with(Value::string);
        let interface = self.interface().await.to_val_or_nothing_with(Value::string);
        let interface_flags = self
            .interface_flags()
            .await
            .map(NMDeviceInterfaceFlags::from_u32_to_str_val)
            .unwrap_or_nothing();
        let ip4_address = self
            .ip4_address()
            .await
            .map(Ipv4Addr::from_u32_to_str_val)
            .unwrap_or_nothing();

        // TODO: ip4_config

        let ip4_connectivity = self
            .ip4_connectivity()
            .await
            .map(NMConnectivityState::from_u32_to_str_val)
            .unwrap_or_nothing();

        // TODO: ip6_config

        let ip6_connectivity = self
            .ip6_connectivity()
            .await
            .map(NMConnectivityState::from_u32_to_str_val)
            .unwrap_or_nothing();
        let ip_interface = self
            .ip_interface()
            .await
            .to_val_or_nothing_with(Value::string);

        // TODO: lldp_neighbors

        let managed = self.managed().await.to_val_or_nothing_with(Value::bool);
        let metered = self
            .metered()
            .await
            .map(NMMetered::from_u32_to_str_val)
            .unwrap_or_nothing();
        let mtu = self
            .mtu()
            .await
            .map(|mtu| value!(int mtu as i64))
            .unwrap_or_nothing();
        let nm_plugin_missing = self
            .nm_plugin_missing()
            .await
            .to_val_or_nothing_with(Value::bool);
        let path = self.path().await.to_val_or_nothing_with(Value::string);
        let physical_port_id = self
            .physical_port_id()
            .await
            .to_val_or_nothing_with(Value::string);

        // TODO: ports

        let real = self.real().await.to_val_or_nothing_with(Value::bool);
        let (state, reason) = match self.state_reason().await.ok() {
            Some((state, reason)) => (
                NMDeviceState::from_u32_to_str_val(state),
                NMDeviceStateReason::from_u32_to_str_val(reason),
            ),
            None => (value!(), value!()),
        };
        let udi = self.udi().await.to_val_or_nothing_with(Value::string);

        value!({
            "capabilities": capabilities,

            // TODO: device_type
            // TODO: dhcp4_config
            // TODO: dhcp6_config

            "driver": value!({
                "name": driver,
                "version": driver_version,
            }),
            "firmware": value!({
                "missing": firmware_missing,
                "version": firmware_version,
            }),
            "hw_address": hw_address,
            "interface": value!({
                "name": interface,
                "flags": interface_flags,
            }),
            "ip4": value!({
                "address": ip4_address,
                // TODO: ip4_config

                "connectivity": ip4_connectivity,
            }),
            "ip6": value!({
                // TODO: ip6_config

                "connectivity": ip6_connectivity
            }),
            "ip_interface": ip_interface,

            // TODO: lldp_neighbors

            "managed": managed,
            "metered": metered,
            "mtu": mtu,
            "nm_plugin_missing": nm_plugin_missing,
            "path": path,
            "physical_port_id": physical_port_id,

            // TODO: ports

            "real": real,
            "state": state,
            "reason": reason,
            "udi": udi
        })
    }
}

pub trait UnwrapOrNothing {
    fn unwrap_or_nothing(self) -> Value;
}

impl UnwrapOrNothing for Option<Value> {
    fn unwrap_or_nothing(self) -> Value {
        self.unwrap_or_else(|| value!())
    }
}

impl<E> UnwrapOrNothing for Result<Value, E> {
    fn unwrap_or_nothing(self) -> Value {
        self.ok().unwrap_or_nothing()
    }
}

pub trait ToValOrNothing<T> {
    fn to_val_or_nothing_with(self, f: impl Fn(T, Span) -> Value) -> Value;
}

impl<T> ToValOrNothing<T> for Option<T> {
    fn to_val_or_nothing_with(self, f: impl Fn(T, Span) -> Value) -> Value {
        match self {
            Some(val) => f(val, Span::unknown()),
            None => Value::nothing(Span::unknown()),
        }
    }
}

impl<T, E> ToValOrNothing<T> for Result<T, E> {
    fn to_val_or_nothing_with(self, f: impl Fn(T, Span) -> Value) -> Value {
        self.ok().to_val_or_nothing_with(f)
    }
}

pub trait ToLabeledResult<T, E>
where
    E: ToLabeledError,
{
    fn to_labeled(self) -> Result<T, LabeledError>;
}

impl<T, E> ToLabeledResult<T, E> for Result<T, E>
where
    E: ToLabeledError,
{
    fn to_labeled(self) -> Result<T, LabeledError> {
        self.map_err(ToLabeledError::to_labeled_error)
    }
}

pub trait ToLabeledError {
    fn to_labeled_error(self) -> LabeledError;
}

impl<E> ToLabeledError for E
where
    E: std::error::Error + std::fmt::Display,
{
    fn to_labeled_error(self) -> LabeledError {
        LabeledError::new(self.to_string())
    }
}
