use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use dirs;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Offset, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use ratatui_core::style::Stylize;
use serde::{Deserialize, Serialize};

use crate::state;

struct Fields {
    username: StringField,
    password: PasswordField,
    tenantid: StringField,
    identity_url: StringField,
}

impl Default for Fields {
    fn default() -> Self {
        Fields {
            username: StringField::new("Username".to_string()),
            password: PasswordField::new("Password".to_string()),
            tenantid: StringField::new("Tenant ID".to_string()),
            identity_url: StringField::new("Identity URL".to_string()),
        }
    }
}

impl From<&Config> for Fields {
    fn from(config: &Config) -> Self {
        Fields {
            username: StringField {
                label: "Username".to_string(),
                value: config.username.clone(),
            },
            password: PasswordField {
                label: "Password".to_string(),
                display_value: "*".repeat(config.password.len()),
                value: config.password.clone(),
            },
            tenantid: StringField {
                label: "Tenant ID".to_string(),
                value: config.tenantid.clone(),
            },
            identity_url: StringField {
                label: "Identity URL".to_string(),
                value: config.identity_url.clone(),
            },
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(skip, default)]
    focus: Focus,

    #[serde(skip, default)]
    pub message: String,

    #[serde(skip, default)]
    fields: Fields,

    pub username: String,
    pub password: String,
    pub tenantid: String,
    pub identity_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            focus: Focus::Username,
            message: String::new(),
            fields: Fields::default(),
            username: String::new(),
            password: String::new(),
            tenantid: String::new(),
            identity_url: String::new(),
        }
    }
}

impl Config {
    pub fn is_valid(&self) -> bool {
        validate(self)
    }

    pub fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical(Constraint::from_lengths([1, 1, 1, 1, 1]));
        let [
            message_area,
            username_area,
            password_area,
            tenantid_area,
            identity_url_area,
        ] = frame.area().layout(&layout);

        let message = Paragraph::new(self.message.clone());
        frame.render_widget(&message, message_area);
        frame.render_widget(&self.fields.username, username_area);
        frame.render_widget(&self.fields.password, password_area);
        frame.render_widget(&self.fields.tenantid, tenantid_area);
        frame.render_widget(&self.fields.identity_url, identity_url_area);

        let cursor_position = match self.focus {
            Focus::Username => username_area.offset(self.fields.username.cursor_offset()),
            Focus::Password => password_area.offset(self.fields.password.cursor_offset()),
            Focus::TenantId => tenantid_area.offset(self.fields.tenantid.cursor_offset()),
            Focus::IdentityUrl => {
                identity_url_area.offset(self.fields.identity_url.cursor_offset())
            }
        };
        frame.set_cursor_position(cursor_position);
    }

    pub fn handle_events(&mut self, event: Option<KeyEvent>) -> state::AppState {
        if let Some(key) = event {
            match key.code {
                KeyCode::Esc => {
                    return state::AppState::Quit;
                }
                KeyCode::Tab => {
                    self.focus = self.focus.next();
                    return state::AppState::Loading;
                }
                KeyCode::Enter => {
                    if self.is_valid() {
                        if let Err(e) = self.save() {
                            self.message = format!("Error saving config: {}", e);
                            return state::AppState::Loading;
                        }
                        return state::AppState::IssueToken {
                            username: self.fields.username.value.clone(),
                            password: self.fields.password.value.clone(),
                            tenantid: self.fields.tenantid.value.clone(),
                            identity_url: self.fields.identity_url.value.clone(),
                        };
                    }

                    self.message = "Please fill in all fields.".to_string();
                    return state::AppState::Loading;
                }
                _ => {
                    match self.focus {
                        Focus::Username => self.fields.username.on_key_press(key),
                        Focus::Password => self.fields.password.on_key_press(key),
                        Focus::TenantId => self.fields.tenantid.on_key_press(key),
                        Focus::IdentityUrl => self.fields.identity_url.on_key_press(key),
                    };
                    return state::AppState::Loading;
                }
            }
        }

        return state::AppState::Loading;
    }

    fn save(&mut self) -> Result<()> {
        let config_path = match dirs::config_dir() {
            Some(path) => path.join("ratatui-sample/config.json"),
            None => return Ok(()),
        };
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.username = self.fields.username.value.clone();
        self.password = self.fields.password.value.clone();
        self.tenantid = self.fields.tenantid.value.clone();
        self.identity_url = self.fields.identity_url.value.clone();

        let config_str = serde_json::to_string(self)?;
        std::fs::write(config_path, config_str)?;

        Ok(())
    }
}

#[derive(Default, PartialEq, Eq)]
enum Focus {
    #[default]
    Username,
    Password,
    TenantId,
    IdentityUrl,
}

impl Focus {
    const fn next(&self) -> Self {
        match self {
            Self::Username => Self::Password,
            Self::Password => Self::TenantId,
            Self::TenantId => Self::IdentityUrl,
            Self::IdentityUrl => Self::Username,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StringField {
    #[serde(skip)]
    label: String,
    pub value: String,
}

impl StringField {
    fn new(label: String) -> Self {
        Self {
            label,
            value: String::new(),
        }
    }

    fn on_key_press(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(c) => self.value.push(c),
            KeyCode::Backspace => {
                self.value.pop();
            }
            _ => {}
        }
    }

    fn cursor_offset(&self) -> Offset {
        let x = (self.label.len() + self.value.len() + 2) as i32;
        Offset::new(x, 0)
    }
}

impl Widget for &StringField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ]);
        let [label_area, value_area] = area.layout(&layout);
        let label = Line::from_iter([self.label.clone(), ": ".to_string()]).bold();
        label.render(label_area, buf);
        self.value.clone().render(value_area, buf);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct PasswordField {
    #[serde(skip)]
    label: String,
    #[serde(skip)]
    display_value: String,
    value: String,
}

impl PasswordField {
    fn new(label: String) -> Self {
        Self {
            label,
            display_value: String::new(),
            value: String::new(),
        }
    }

    fn on_key_press(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(c) => {
                self.value.push(c);
                self.display_value.push('*');
            }
            KeyCode::Backspace => {
                self.value.pop();
                self.display_value.pop();
            }
            _ => {}
        }
    }

    fn cursor_offset(&self) -> Offset {
        let x = (self.label.len() + self.value.len() + 2) as i32;
        Offset::new(x, 0)
    }
}

impl Widget for &PasswordField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ]);
        let [label_area, value_area] = area.layout(&layout);
        let label = Line::from_iter([self.label.clone(), ": ".to_string()]).bold();
        label.render(label_area, buf);
        self.display_value.clone().render(value_area, buf);
    }
}

pub fn load() -> Config {
    let config_path = match dirs::config_dir() {
        Some(path) => path.join("ratatui-sample/config.json"),
        None => return Config::default(),
    };
    if !config_path.exists() {
        return Config::default();
    }

    let config_str = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(_) => return Config::default(),
    };
    let mut config: Config = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse config: {}", e.to_string().red());
            return Config::default();
        }
    };
    config.fields = Fields::from(&config);

    if validate(&config) {
        config
    } else {
        eprintln!("Invalid config: Missing required fields");
        Config::default()
    }
}

fn validate(config: &Config) -> bool {
    !config.fields.username.value.is_empty()
        && !config.fields.password.value.is_empty()
        && !config.fields.tenantid.value.is_empty()
        && !config.fields.identity_url.value.is_empty()
}
