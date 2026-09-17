use std::{env, num::ParseIntError};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("environment variable {name} is not set")]
    MissingVariable { name: &'static str },

    #[error("environment variable {name} contains non-Unicode data")]
    NonUnicodeVariable { name: &'static str },

    #[error("environment variable APP_PORT value {value:?} is not a valid port")]
    InvalidPort {
        value: String,
        #[source]
        source: ParseIntError,
    },
}

pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_env_values(
            env::var("APP_HOST"),
            env::var("APP_PORT"),
            env::var("DATABASE_URL"),
        )
    }

    fn from_env_values(
        host: Result<String, env::VarError>,
        port: Result<String, env::VarError>,
        database_url: Result<String, env::VarError>,
    ) -> Result<Self, ConfigError> {
        let host: String = optional_variable(host, "APP_HOST", "0.0.0.0")?;
        let port_value: String = optional_variable(port, "APP_PORT", "4000")?;
        let port: u16 = port_value.parse::<u16>().map_err(|source: ParseIntError| {
            ConfigError::InvalidPort {
                value: port_value,
                source,
            }
        })?;
        let database_url = required_variable(database_url, "DATABASE_URL")?;

        Ok(Self {
            host,
            port,
            database_url,
        })
    }
}

fn optional_variable(
    value: Result<String, env::VarError>,
    name: &'static str,
    default: &str,
) -> Result<String, ConfigError> {
    match value {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Ok(default.to_string()),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::NonUnicodeVariable { name }),
    }
}

fn required_variable(
    value: Result<String, env::VarError>,
    name: &'static str,
) -> Result<String, ConfigError> {
    match value {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(ConfigError::MissingVariable { name }),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::NonUnicodeVariable { name }),
    }
}

#[cfg(test)]
mod tests {
    use std::{env::VarError, ffi::OsString};

    use super::{Config, ConfigError};

    #[test]
    fn rejects_an_invalid_port() {
        let result = Config::from_env_values(
            Ok("127.0.0.1".to_string()),
            Ok("not-a-port".to_string()),
            Ok("postgres://localhost/users".to_string()),
        );

        assert!(matches!(
            result,
            Err(ConfigError::InvalidPort { value, .. }) if value == "not-a-port"
        ));
    }

    #[test]
    fn rejects_a_missing_database_url() {
        let result: Result<Config, ConfigError> = Config::from_env_values(
            Err(VarError::NotPresent),
            Err(VarError::NotPresent),
            Err(VarError::NotPresent),
        );

        assert!(matches!(
            result,
            Err(ConfigError::MissingVariable {
                name: "DATABASE_URL"
            })
        ));
    }

    #[test]
    fn rejects_non_unicode_configuration() {
        let result: Result<Config, ConfigError> = Config::from_env_values(
            Err(VarError::NotUnicode(OsString::from("invalid-host"))),
            Err(VarError::NotPresent),
            Ok("postgres://localhost/users".to_string()),
        );

        assert!(matches!(
            result,
            Err(ConfigError::NonUnicodeVariable { name: "APP_HOST" })
        ));
    }

    #[test]
    fn defaults_optional_host_and_port_when_absent() {
        let config: Config = Config::from_env_values(
            Err(VarError::NotPresent),
            Err(VarError::NotPresent),
            Ok("postgres://localhost/users".to_string()),
        )
        .expect("valid configuration should be accepted");

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 4000);
    }
}
