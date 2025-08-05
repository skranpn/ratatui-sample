use color_eyre::eyre::Result;
use crossterm::event::{self};
use ratatui::DefaultTerminal;

use crate::config;
use crate::openstack::server::Server;
use crate::openstack::token;
use crate::state;

pub struct App {
    token: String,
    endpoints: Vec<token::Endpoint>,
    config: config::Config,
    state: state::AppState,
}

impl App {
    pub fn new() -> Self {
        let config = config::load();
        let mut state = state::AppState::Loading;
        if config.is_valid() {
            state = state::AppState::IssueToken {
                username: config.username.clone(),
                password: config.password.clone(),
                tenantid: config.tenantid.clone(),
                identity_url: config.identity_url.clone(),
            }
        }
        Self {
            token: String::new(),
            endpoints: Vec::new(),
            config: config,
            state: state,
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.is_running() {
            match self.state {
                state::AppState::Loading => {
                    let _ = terminal.draw(|frame| self.config.render(frame));
                    self.state = self
                        .config
                        .handle_events(event::read()?.as_key_press_event());
                }
                state::AppState::IssueToken {
                    ref username,
                    ref password,
                    ref tenantid,
                    ref identity_url,
                } => {
                    match token::issue_token(
                        username.clone(),
                        password.clone(),
                        tenantid.clone(),
                        identity_url.clone(),
                    )
                    .await
                    {
                        Ok(res) => {
                            self.token = res.token;
                            self.endpoints = res.endpoints;
                            self.state = state::AppState::Server;
                        }
                        Err(e) => {
                            self.config.message = format!("Error issuing token: {}", e);
                            self.state = state::AppState::Loading;
                        }
                    }
                }
                state::AppState::Server => {
                    let server = Server::new("http://localhost:5000".to_string());
                    self.state = server.run(&mut terminal).await?;
                }
                state::AppState::Quit => {
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        self.state != state::AppState::Quit
    }
}
