use crate::state::AppState;
use anyhow::{Result, anyhow};
use crossterm::event::{Event, EventStream, KeyCode};
use ratatui::style::{Style, Stylize};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, HighlightSpacing, Row, StatefulWidget, Table, TableState, Widget},
};
use reqwest::Client;
use serde::Deserialize;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio_stream::StreamExt;

pub struct Server {
    url: String,
    widget: ServerListWidget,
    should_quit: bool,
}

impl Server {
    const FRAMES_PER_SECOND: f32 = 60.0;
    pub fn new(url: String) -> Self {
        Self {
            url: url,
            widget: ServerListWidget::default(),
            should_quit: false,
        }
    }

    pub async fn run(
        mut self,
        terminal: &mut DefaultTerminal,
    ) -> color_eyre::eyre::Result<AppState> {
        self.widget.run(self.url.clone());
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.render(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event),
            }
        }

        Ok(AppState::Quit)
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [title_area, body_area] = frame.area().layout(&layout);

        let title = Line::from("Servers").centered().bold();
        frame.render_widget(title, title_area);
        frame.render_widget(&self.widget, body_area);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Some(key) = event.as_key_press_event() {
            match key.code {
                KeyCode::Esc => self.should_quit = true,
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServerListWidget {
    state: Arc<RwLock<ServerListState>>,
}

#[derive(Debug, Default)]
struct ServerListState {
    servers: Vec<ServerState>,
    loading_state: LoadingState,
    table_state: TableState,
}

#[derive(Debug, Clone)]
struct ServerState {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error(String),
}

impl ServerListWidget {
    fn run(&self, url: String) {
        let this = self.clone();
        tokio::spawn(this.fetch_servers(url));
    }

    async fn fetch_servers(self, url: String) {
        self.set_loading_state(LoadingState::Loading);
        match list_servers_detail(url).await {
            Ok(resp) => self.on_load(&resp),
            Err(err) => self.on_err(&err),
        }
    }

    fn on_load(&self, servers: &ServersDetail) {
        let servers = servers.servers.iter().map(|s| ServerState {
            id: s.id.clone(),
            name: s.name.clone(),
        });
        let mut state = self.state.write().unwrap();
        state.loading_state = LoadingState::Loaded;
        state.servers.extend(servers);
        if !state.servers.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn on_err(&self, err: &anyhow::Error) {
        self.set_loading_state(LoadingState::Error(err.to_string()));
    }

    fn set_loading_state(&self, state: LoadingState) {
        self.state.write().unwrap().loading_state = state;
    }
}

impl Widget for &ServerListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let loading_state = Line::from(format!("{:?}", state.loading_state)).right_aligned();
        let block = Block::bordered()
            .title("Servers")
            .title(loading_state)
            .title_bottom("j/k to scroll, Esc to quit");

        let rows = state.servers.iter();
        let widths = [
            Constraint::Length(36),
            Constraint::Fill(1),
            Constraint::Max(49),
        ];
        let table = Table::new(rows, widths)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>")
            .row_highlight_style(Style::new().on_blue());

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

// token 発行
async fn list_servers_detail(url: String) -> Result<ServersDetail> {
    let client = Client::new();
    let url = format!("{}/servers/detail", url);
    let resp = client.get(&url).send().await?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(anyhow!("Unexpected status: {}", resp.status()));
    }

    let body = resp.json::<ServersDetail>().await?;

    Ok(body)
}

#[derive(Deserialize, Debug)]
struct ServersDetail {
    servers: Vec<Server_>,
}

#[derive(Deserialize, Debug)]
struct Server_ {
    id: String,
    name: String,
    status: String,
    #[serde(rename = "OS-EXT-STS:task_state")]
    task_state: Option<String>,
    #[serde(rename = "OS-EXT-STS:vm_state")]
    vm_state: String,
}

impl From<&ServerState> for Row<'_> {
    fn from(value: &ServerState) -> Self {
        let server = value.clone();
        Row::new(vec![server.id, server.name])
    }
}
