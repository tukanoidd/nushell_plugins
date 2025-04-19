use std::ops::Deref;

use nu_protocol::{LabeledError, PipelineData, Record, Span, Value};
use rusty_network_manager::{ActiveProxy, ConnectionProxy, NetworkManagerProxy};

#[derive(derive_more::Deref)]
pub struct NMConnection<'a> {
    proxy: NetworkManagerProxy<'a>,
}

impl<'a> NMConnection<'a> {
    pub async fn new(zbus_connection: &'a zbus::Connection) -> Result<Self, LabeledError> {
        let proxy = NetworkManagerProxy::new(zbus_connection)
            .await
            .to_labeled()?;

        Ok(Self { proxy })
    }

    pub async fn version(&self) -> Result<PipelineData, LabeledError> {
        Ok(PipelineData::Value(
            Value::string(self.deref().version().await.to_labeled()?, Span::unknown()),
            None,
        ))
    }

    pub async fn status(
        &self,
        zbus_connection: &zbus::Connection,
    ) -> Result<PipelineData, LabeledError> {
        let active_connections = self.active_connections().await.to_labeled()?;
        let active_connection_records: Vec<_> =
            futures::future::join_all(active_connections.into_iter().map(
                |connection_path| async move {
                    let active_proxy =
                        ActiveProxy::new_from_path(connection_path.clone(), zbus_connection)
                            .await
                            .ok()?;

                    let default = active_proxy.default().await.to_val_or_nothing(Value::bool);
                    let default6 = active_proxy.default6().await.to_val_or_nothing(Value::bool);

                    // TODO: devices
                    // TODO: dhcp4_config
                    // TODO: dhcp6_config

                    let id = active_proxy.id().await.to_val_or_nothing(Value::string);

                    // TODO: ip4_config
                    // TODO: ip6_config
                    // TODO: master
                    // TODO: specific_object

                    // TODO: better presentation of state
                    let state = active_proxy
                        .state()
                        .await
                        .map(|s| s as i64)
                        .to_val_or_nothing(Value::int);
                    // TODO: proper presentation of flags
                    let state_flags = active_proxy
                        .state_flags()
                        .await
                        .map(|f| f as i64)
                        .to_val_or_nothing(Value::int);

                    let type_ = active_proxy.type_().await.to_val_or_nothing(Value::string);
                    let uuid = active_proxy.uuid().await.to_val_or_nothing(Value::string);

                    let vpn = match active_proxy.vpn().await.ok().unwrap_or_default() {
                        true => {
                            match ConnectionProxy::new_from_path(connection_path, zbus_connection)
                                .await
                                .ok()
                            {
                                Some(connection_proxy) => {
                                    let banner = connection_proxy
                                        .banner()
                                        .await
                                        .to_val_or_nothing(Value::string);
                                    // TODO: better presentation of state
                                    let vpn_state = connection_proxy
                                        .vpn_state()
                                        .await
                                        .map(|s| s as i64)
                                        .to_val_or_nothing(Value::int);

                                    Value::record(
                                        Record::from_iter([
                                            ("banner".into(), banner),
                                            ("vpn_state".into(), vpn_state),
                                        ]),
                                        Span::unknown(),
                                    )
                                }
                                None => Value::nothing(Span::unknown()),
                            }
                        }
                        false => Value::nothing(Span::unknown()),
                    };

                    Some(Value::record(
                        Record::from_iter(
                            [
                                ("default", default),
                                ("default6", default6),
                                // TODO: devices
                                // TODO: dhcp4_config
                                // TODO: dhcp6_config
                                ("id", id),
                                // TODO: ip4_config
                                // TODO: ip6_config
                                // TODO: master
                                // TODO: specific_object
                                ("state", state),
                                ("state_flags", state_flags),
                                ("type", type_),
                                ("uuid", uuid),
                                ("vpn", vpn),
                            ]
                            .into_iter()
                            .map(|(key, val)| (key.into(), val)),
                        ),
                        Span::unknown(),
                    ))
                },
            ))
            .await
            .into_iter()
            .flatten()
            .collect();

        Ok(PipelineData::Value(
            Value::record(
                Record::from_iter([(
                    "active_connections".into(),
                    Value::list(active_connection_records, Span::unknown()),
                )]),
                Span::unknown(),
            ),
            None,
        ))
    }
}

pub trait ToValOrNothing<T> {
    fn to_val_or_nothing(self, f: impl Fn(T, Span) -> Value) -> Value;
}

impl<T> ToValOrNothing<T> for Option<T> {
    fn to_val_or_nothing(self, f: impl Fn(T, Span) -> Value) -> Value {
        match self {
            Some(val) => f(val, Span::unknown()),
            None => Value::nothing(Span::unknown()),
        }
    }
}

impl<T, E> ToValOrNothing<T> for Result<T, E> {
    fn to_val_or_nothing(self, f: impl Fn(T, Span) -> Value) -> Value {
        self.ok().to_val_or_nothing(f)
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
