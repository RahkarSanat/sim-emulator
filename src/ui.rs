use std::{
    collections::VecDeque,
    io,
    ops::Not,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use crate::sim868::{GnssConfiguration, Sim868};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    prelude::*,
    symbols::scrollbar,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Terminal,
};

pub fn run_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    rx: Receiver<String>,
    tx: Sender<String>,
) -> io::Result<()> {
    let mut selected_button: usize = 0;
    // let button_states = &mut [State::Selected, State::Normal, State::Normal];

    let title_block = Block::default()
        .border_style(Style::default())
        .borders(Borders::ALL);
    let selected_block = Block::default()
        .border_style(Style::default().fg(Color::LightGreen))
        .borders(Borders::ALL);

    let mut text_area = ScrollableTextArea::new(1000);
    // for i in 0..110 {
    //     text_area.add_line(format!("{}: test\n", i));
    // }
    text_area.set_content_length(1000);

    let mut sim_device = Sim868::new(true, GnssConfiguration::default());
    let _gnss_tx = sim_device.start_gnss(tx.clone());

    loop {
        {
            terminal.draw(|frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(95), Constraint::Percentage(5)])
                    .split(frame.size());
                let main_screen = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(chunks[0]);
                frame.render_widget(
                    Paragraph::new(
                        "ALT + q: quit\tToggle Gnss Power: ALT=h\tToggle gsm power: ALT+g",
                    )
                    .style(Style::default().bg(Color::Green)),
                    chunks[1],
                );

                // frame.render_widget(text_area.widget(), main_screen[0]);
                // text_area.add_line("test".to_owned());
                text_area.render(frame, main_screen[0]);
            })?;
        }

        if let Ok(line) = rx.recv_timeout(Duration::from_millis(10)) {
            // process receiving data from sim

            let data = sim_device.process_at(&line).unwrap();
            let mut answer = String::new();
            for at in data {
                answer += &format!("◁◁  {} -> ", &line);
                if let Ok(_) = tx.send(at.clone()) {
                    answer += &format!("▶ {}", &at);
                } else {
                    text_area.add_line("failed to send this command ->".to_string());
                }
            }
            text_area.add_line(answer);
        }

        if !event::poll(Duration::from_millis(16))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::ALT {
                        break;
                    } else if key.code == KeyCode::Up {
                        text_area.scroll_up(4);
                    } else if key.code == KeyCode::Down {
                        text_area.scroll_down(4);
                    } else if key.code == KeyCode::Char('g') && key.modifiers == KeyModifiers::ALT {
                        sim_device.power = !sim_device.power;
                    } else if key.code == KeyCode::Char('h') && key.modifiers == KeyModifiers::ALT {
                        let d = !sim_device.gnss.lock().unwrap().power;
                        sim_device.gnss.lock().unwrap().power = d;
                    }
                }
            }
            // Event::Mouse(mouse) => handle_mouse_event(mouse, button_states, &mut selected_button),
            _ => (),
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ScrollableTextArea {
    buffer: VecDeque<String>,
    vertical_scroll: usize,
    capacity: usize,
    horizontal_scroll: usize,
    scrollbar_state: ScrollbarState,
    content_length: usize,
}

impl ScrollableTextArea {
    pub fn new(line_capacity: usize) -> ScrollableTextArea {
        ScrollableTextArea {
            buffer: VecDeque::with_capacity(line_capacity),
            vertical_scroll: 0,
            horizontal_scroll: 0,
            capacity: line_capacity,
            scrollbar_state: ScrollbarState::default(),
            content_length: 0,
        }
    }
    pub fn scroll_up(&mut self, to: usize) -> bool {
        // if self.vertical_scroll > self.buffer.len() - 10 {
        //     return false;
        // }
        self.vertical_scroll = self.vertical_scroll.saturating_add(to);
        self.scrollbar_state = self.scrollbar_state.position(self.vertical_scroll);
        true
    }
    pub fn scroll_down(&mut self, to: usize) -> bool {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(to);
        self.scrollbar_state = self.scrollbar_state.position(self.vertical_scroll);
        true
    }

    pub fn set_content_length(&mut self, length: usize) {
        self.content_length = length;
        self.scrollbar_state.content_length(length);
    }

    pub fn add_line(&mut self, line: String) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        } else {
            self.scrollbar_state
                .content_length(self.content_length.saturating_add(1));
        }
        self.buffer.push_back(line);
    }
    pub fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let n = self.buffer.len();
        let mut screen_content = String::new();
        for l in n
            .saturating_sub(area.height as usize)
            .saturating_sub(self.vertical_scroll)
            ..n.saturating_sub(1).saturating_sub(self.vertical_scroll)
        {
            screen_content += &self.buffer[l];
            screen_content.push('\n');
        }
        frame.render_widget(
            Paragraph::new(screen_content).block(
                Block::default()
                    .border_style(Style::default())
                    .borders(Borders::ALL),
            ),
            area,
        );
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL),
            area.inner(&Margin {
                horizontal: 0,
                vertical: 0,
            }),
            &mut self.scrollbar_state,
        )
    }
}
