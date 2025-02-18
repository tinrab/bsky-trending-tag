use anyhow::Result;
use async_nats::{Client, ConnectOptions};

use crate::config::NatsBrokerConfig;

#[derive(Clone)]
pub struct NatsBroker {
    client: Client,
    config: NatsBrokerConfig,
}

impl NatsBroker {
    pub async fn connect(config: &NatsBrokerConfig) -> Result<NatsBroker> {
        let mut conn_options = ConnectOptions::new();
        if let Some((user, password)) = config
            .user
            .clone()
            .and_then(|user| Some((user, config.password.clone()?)))
        {
            conn_options = conn_options.user_and_password(user, password);
        }
        let client = conn_options.connect(&config.address).await?;

        Ok(NatsBroker {
            client,
            config: config.clone(),
        })
    }

    pub async fn publish(&self, subject: &str, message: Vec<u8>) -> Result<()> {
        let subject = if let Some(subject_prefix) = self.config.subject_prefix.as_ref() {
            format!("{subject_prefix}:{subject}")
        } else {
            subject.to_string()
        };

        self.client.publish(subject, message.into()).await?;
        self.client.flush().await?;

        Ok(())
    }
}
