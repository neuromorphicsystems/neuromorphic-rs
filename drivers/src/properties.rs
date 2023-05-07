#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Camera<Configuration> {
    pub name: &'static str,
    pub width: u16,
    pub height: u16,
    pub default_configuration: Configuration,
}
