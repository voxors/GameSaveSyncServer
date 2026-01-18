use once_cell::sync::Lazy;
use regex::Regex;
use std::{collections::HashMap, error::Error};

use crate::{DATABASE, datatype_endpoint::ConfigurationForm};

pub const MAX_SAVE_PER_GAME_INFO: ConfigurationInfo = ConfigurationInfo {
    id: "max_save_per_game",
    name: "Number of save to keep per game",
    max: Some(999),
    min: Some(1),
    step: Some(1),
    pattern: None,
};

pub static CONFIG_MAP: Lazy<HashMap<&'static str, ConfigurationInfo<'static>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(MAX_SAVE_PER_GAME_INFO.id, MAX_SAVE_PER_GAME_INFO);
    map
});

#[derive(Clone, Copy)]
pub struct ConfigurationInfo<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub max: Option<u32>,
    pub min: Option<u32>,
    pub step: Option<u32>,
    pub pattern: Option<&'a str>,
}

impl<'a> ConfigurationInfo<'a> {
    pub fn validate(&self, raw: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.min.is_some() || self.max.is_some() || self.step.is_some() {
            let value: u32 = raw.parse::<u32>()?;
            if let Some(min) = self.min
                && value < min
            {
                return Err(format!("{} must be at least {}", self.name, min).into());
            }
            if let Some(max) = self.max
                && value > max
            {
                return Err(format!("{} must be at most {}", self.name, max).into());
            }
            if let Some(step) = self.step
                && !(value - self.min.unwrap_or(0)).is_multiple_of(step)
            {
                return Err(format!("{} must be a multiple of {}", self.name, step).into());
            }
        }
        if let Some(patern) = self.pattern {
            let re = Regex::new(patern)?;
            if !re.is_match(raw) {
                return Err(format!("{} does not match the required pattern", self.name).into());
            }
        }
        Ok(())
    }

    pub fn get_value_in_db(
        &self,
    ) -> Result<Option<ConfigurationForm>, Box<dyn Error + Send + Sync>> {
        match DATABASE.get_configuration_value(self.id) {
            Ok(maybe_db_config) => match maybe_db_config {
                Some(db_config) => Ok(Some(ConfigurationForm {
                    value: db_config.value,
                })),
                None => Ok(None),
            },
            Err(err) => Err(err),
        }
    }

    pub fn update_value_in_db(&self, value: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        match DATABASE.update_configuration_value(self.id, value) {
            Ok(()) => Ok(()),
            Err(err) => {
                tracing::error!("Error updating configuration: {}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::interface::GameDatabase;

    #[test]
    fn test_configs_exist_in_default_db() {
        let db = GameDatabase::new(&format!(
            "file:{}?mode=memory&cache=shared",
            uuid::Uuid::new_v4()
        ));

        for cfg in CONFIG_MAP.values() {
            let maybe_db_configuration = db
                .get_configuration_value(cfg.id)
                .expect("DB query failed for configuration value");
            let db_configuration = maybe_db_configuration
                .unwrap_or_else(|| panic!("Missing config `{}` in the database", cfg.id));
            cfg.validate(&db_configuration.value).unwrap_or_else(|err| {
                panic!("validation failed for `{} with err: {}`", cfg.id, err)
            })
        }
    }
}
