use serde::{Deserialize, Serialize};


#[derive(Deserialize, Debug)]
pub struct OneToTenConfig {
    pub text: String,
    pub label_at_one: String,
    pub label_at_ten: String,
    pub required: bool,
}

#[derive(Deserialize, Debug)]
pub struct EssayConfig {
    text: String,
    required: bool,
}

#[derive(Deserialize, Debug)]
pub struct RadioChoicesConfig {
    pub text: String,
    pub choices: Vec<String>,
    pub required: bool,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WidgetConfig {
    OneToTen(OneToTenConfig),
    Essay(EssayConfig),
    RadioChoices(RadioChoicesConfig),
}

impl WidgetConfig {
    pub fn is_required(&self) -> bool {
        match self {
            WidgetConfig::OneToTen(c) => c.required,
            WidgetConfig::Essay(c) => c.required,
            WidgetConfig::RadioChoices(c) => c.required,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            WidgetConfig::OneToTen(c) => &c.text,
            WidgetConfig::Essay(c) => &c.text,
            WidgetConfig::RadioChoices(c) => &c.text,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct FormConfig {
    pub title: String,
    pub widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Clone)]
pub enum WidgetState {
    OneToTen(Option<u8>),
    Essay(String),
    RadioChoices(Option<String>),
}

impl WidgetState {
    pub fn is_answered(&self) -> bool {
        match self {
            WidgetState::OneToTen(Some(_)) => true,
            WidgetState::Essay(s) => !s.trim().is_empty(),
            WidgetState::RadioChoices(Some(_)) => true,
            _ => false,
        }
    }
}
impl std::fmt::Display for WidgetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetState::OneToTen(Some(val)) => write!(f, "{}", val),
            WidgetState::RadioChoices(Some(choice)) => write!(f, "{}", choice),
            WidgetState::Essay(text) => write!(f, "{}", text),
            WidgetState::OneToTen(None) | WidgetState::RadioChoices(None) => write!(f, ""),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct FormSubmission {
    pub survey_id: String,
    pub user_name: String,
    pub user_xuid: String,
    pub map_name: String,
    pub game_timestamp: f32,
    pub submission_timestamp: u64,
    pub answers: std::collections::BTreeMap<String, String>,

    #[serde(flatten)]
    pub extra_data: std::collections::BTreeMap<String, serde_json::Value>,
}
