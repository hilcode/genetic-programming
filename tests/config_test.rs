use gp_engine::test_support::depth_value;
use gp_engine::test_support::population_size_value;
use gp_engine::GpConfig;
use gp_engine::RawConfig;
use std::fs;

#[test]
fn test_config_defaults() {
    let raw: RawConfig = RawConfig::with_defaults();
    let config: GpConfig = raw.try_into().expect("defaults should be valid");

    // Verify defaults match expected values
    assert_eq!(population_size_value(&config.population_size), 100);
    assert_eq!(config.generations, 50);
    assert_eq!(depth_value(&config.max_depth), 6);
}

#[test]
fn test_config_from_toml() {
    let contents: String = fs::read_to_string("tests/fixtures/valid.conf")
        .expect("Failed to read valid.conf");
    let file_raw: RawConfig = toml::from_str(&contents)
        .expect("Failed to parse TOML");

    let merged: RawConfig = RawConfig::with_defaults().merge(file_raw);
    let config: GpConfig = merged.try_into().expect("should be valid");

    assert_eq!(population_size_value(&config.population_size), 200);
    assert_eq!(config.generations, 75);
    assert_eq!(depth_value(&config.max_depth), 8);
}

#[test]
fn test_config_cli_override() {
    let file_contents: String = fs::read_to_string("tests/fixtures/valid.conf")
        .expect("Failed to read valid.conf");
    let file_raw: RawConfig = toml::from_str(&file_contents)
        .expect("Failed to parse TOML");

    // Simulate CLI override
    let cli_raw: RawConfig = RawConfig {
        population_size: Some(300),
        generations: Some(100),
        ..Default::default()
    };

    let merged: RawConfig = RawConfig::with_defaults()
        .merge(file_raw)
        .merge(cli_raw);
    let config: GpConfig = merged.try_into().expect("should be valid");

    // CLI should override file
    assert_eq!(population_size_value(&config.population_size), 300);
    assert_eq!(config.generations, 100);
    // File value should be used for max_depth
    assert_eq!(depth_value(&config.max_depth), 8);
}

#[test]
fn test_config_validation_error() {
    let contents: String = fs::read_to_string("tests/fixtures/invalid.conf")
        .expect("Failed to read invalid.conf");
    let file_raw: RawConfig = toml::from_str(&contents)
        .expect("Failed to parse TOML");

    let merged: RawConfig = RawConfig::with_defaults().merge(file_raw);
    let result = GpConfig::try_from(merged);

    // Should fail because population_size = 0 is invalid
    assert!(result.is_err());
    if let Err(error) = result {
        let error_message: String = error.to_string();
        assert!(error_message.contains("population_size"));
    }
}

#[test]
fn test_empty_config_uses_defaults() {
    let empty: RawConfig = RawConfig::default();
    let merged: RawConfig = RawConfig::with_defaults().merge(empty);
    let config: GpConfig = merged.try_into().expect("defaults should be valid");

    assert_eq!(population_size_value(&config.population_size), 100);
    assert_eq!(config.generations, 50);
}
