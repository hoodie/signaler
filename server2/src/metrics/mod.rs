use prometheus::{IntGauge, Opts, Registry};
use tracing::log;
use xactor::Service;

mod actor;
pub mod command {
    use prometheus::{IntGauge, Registry};

    #[xactor::message(result = "Registry")]
    pub struct GetRegistry;

    #[xactor::message(result = "Option<IntGauge>")]
    pub struct AddGauge {
        pub name: String,
        pub help: String,
    }
}

#[derive(Debug, Default)]
pub struct MetricsService {
    registry: Registry,
}

impl MetricsService {
    pub async fn get_registry() -> xactor::Result<Registry> {
        let registry = Self::from_registry().await?.call(self::command::GetRegistry).await?;
        Ok(registry)
    }

    pub async fn get_gauge(name: &str, help: &str) -> xactor::Result<Option<IntGauge>> {
        let gauge = Self::from_registry()
            .await?
            .call(self::command::AddGauge {
                name: name.into(),
                help: help.into(),
            })
            .await?;
        Ok(gauge)
    }

    pub fn add_gauge(&self, name: &str, help: &str) -> Option<IntGauge> {
        log::trace!("creating new gauge");
        let gauge = match IntGauge::with_opts(Opts::new(name, help)) {
            Ok(gauge) => gauge,
            Err(err) => {
                log::error!("cannot instantitate gauge {:?} {}", (name, help), err);
                return None;
            }
        };

        if let Err(error) = self.registry.register(Box::new(gauge.clone())) {
            log::error!("cannot register gauge {}", error);
        }

        Some(gauge)
    }
}
