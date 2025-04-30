use iced::Event::Keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::{Event, Key, Modifiers};
use iced::theme::Style;
use iced::widget::Grid;
use iced::{Border, Length, Size, Task, Theme};
use iced::{
    Color, Element, Font,
    Length::Fill,
    Subscription,
    widget::{container, text},
    window,
};
use mouce::{Mouse, MouseActions};

pub fn main() -> iced::Result {
    iced::application(
        MouseApplication::boot,
        MouseApplication::update,
        MouseApplication::view,
    )
    .window(MouseApplication::window())
    .subscription(MouseApplication::keyboard_subscription)
    .style(MouseApplication::style)
    .run()
}

#[derive(Default)]
struct MouseApplication {
    device: Mouse,
    keypoints: Vec<Cell>,
    selected_cell: Option<Cell>,
    sub_chars: Vec<char>,
    size: Option<Size>,
    scale: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    first_char: char,
    label: &'static str,
    sub_select: char,
    sub_cells: bool,
    selected: bool,
    index: u32,
    height: f32,
}

impl Cell {
    fn new(first_char: char, sub_select: char, index: u32, label: &'static str) -> Self {
        Self {
            selected: false,
            sub_cells: false,
            height: 0.0,
            first_char,
            sub_select,
            index,
            label,
        }
    }
}

fn sub_cell_factory(ch: char) -> Element<'static, Message> {
    container(text(ch).size(12).font(Font::MONOSPACE).center())
        .center_x(Fill)
        .center_y(Fill)
        .style(move |theme: &Theme| {
            let palette = theme.palette();
            container::Style {
                border: Border {
                    width: 1.0,
                    color: palette.danger,
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

fn cell_factory(cell: Cell) -> Element<'static, Message> {
    if cell.sub_cells {
        let sub_cells = (b'A'..=b'R')
            .chain(b'a'..=b'z')
            .chain(b'0'..=b'9')
            .map(|point| sub_cell_factory(point as char))
            .collect();
        let sub_grid = Grid::from_vec(sub_cells)
            .columns(9)
            .height(cell.height * 2.0);

        container(sub_grid.spacing(1))
            .style(move |theme| container_style(theme, cell))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    } else {
        container(text(cell.label).size(54).font(Font::MONOSPACE))
            .style(move |theme| container_style(theme, cell))
            .padding(10)
            .center(Length::Fill)
            .into()
    }
}

fn container_style(theme: &iced::Theme, cell: Cell) -> container::Style {
    let palette = theme.palette();
    let color = if cell.selected {
        palette.danger
    } else {
        palette.background
    };

    container::Style {
        border: Border {
            width: 2.0,
            color,
            ..Border::default()
        },
        ..container::Style::default()
    }
}

impl MouseApplication {
    fn boot() -> (Self, Task<Message>) {
        let keypoints = vec![
            "12", "23", "34", "45", "56", "67", "78", "89", "90", "qw", "we", "et", "yu", "ui",
            "io", "op", "p[", "[]", "]a", "as", "sd", "df", "fg", "gh", "hj", "l;", ";'", "zx",
            "xc", "cv", "vb", "bn", "nm", "m,", ",.", "./",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, val)| {
            let mut chars = val.chars();

            Cell::new(chars.next().unwrap(), chars.next().unwrap(), i as u32, val)
        })
        .collect();

        let sub_chars = (b'A'..=b'R')
            .chain(b'a'..=b'z')
            .chain(b'0'..=b'9')
            .map(char::from)
            .collect::<Vec<char>>();

        (
            Self {
                device: Mouse::new(),
                size: None,
                scale: None,
                selected_cell: None,
                keypoints,
                sub_chars,
            },
            Task::batch([
                window::get_latest()
                    .and_then(window::get_size)
                    .map(Message::WindowSize),
                window::get_latest()
                    .and_then(window::get_scale_factor)
                    .map(Message::WindowScale),
            ]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPressed(ch) => match self.selected_cell {
                Some(mut selected) => {
                    println!("keypressed: {}", ch);
                    println!("selected: {:?}", selected);

                    if selected.sub_cells {
                        println!("should sub grid char");

                        if let Some(sub_cell_idx) = self.sub_chars.iter().position(|&c| c == ch) {
                            let Size { width, height } = self.size.unwrap();
                            let row = selected.index / 9;
                            let col = selected.index % 9;

                            let x1 = (width / 9.0) * col as f32 / 2.0;
                            let y1 = (height / 4.0) * row as f32 / 2.0;

                            let inner_row = sub_cell_idx / 9;
                            let inner_col = sub_cell_idx % 9;

                            let x2 = (width / 9.0 / 9.0) * inner_col as f32 / 2.0;
                            let y2 = (height / 4.0 / 6.0) * inner_row as f32 / 2.0;

                            let x2_pad = x1 + x2;
                            let y2_pad = y1 + y2;

                            let x2_offet = (width / 9.0 / 9.0 / 2.0 / 2.0) as i32;
                            let y2_offet = (height / 4.0 / 6.0 / 2.0 / 2.0) as i32;

                            let _ = self
                                .device
                                .move_to(x2_pad as i32 + x2_offet, y2_pad as i32 + y2_offet);

                            println!(
                                "row:{row} col:{col} cell: {x1}x{y1} {inner_row}x{inner_col} {inner_col} sub: {x2}x{y2}"
                            );

                            return window::get_latest().and_then(window::close);
                        }
                    } else if ch == selected.sub_select {
                        println!("should select sub grid");
                        let new_keypoints = self.keypoints.clone();

                        self.keypoints = new_keypoints
                            .into_iter()
                            .map(|mut c| {
                                c.sub_cells = c.index == selected.index;
                                c
                            })
                            .collect::<Vec<Cell>>();
                        selected.sub_cells = true;
                        self.selected_cell = Some(selected);
                    }

                    if ch == selected.first_char {
                        let Size { width, height } = self.size.unwrap();
                        let row = selected.index / 9;
                        let col = selected.index % 9;

                        let x = (width / 9.0) * col as f32 / 2.0;
                        let y = (height / 4.0) * row as f32 / 2.0;

                        let x_pad = width / 9.0 / 2.0 / 2.0;
                        let y_pad = height / 4.0 / 2.0 / 2.0;

                        let x_final = (x + x_pad) as i32;
                        let y_final = (y + y_pad) as i32;

                        let _ = self.device.move_to(x_final, y_final);

                        window::get_latest().and_then(window::close)
                    } else {
                        Task::none()
                    }
                }
                None => {
                    if let Some(cell) = self.keypoints.clone().iter().find(|k| k.first_char == ch) {
                        let new_keypoints = self.keypoints.clone();

                        self.keypoints = new_keypoints
                            .into_iter()
                            .map(|mut c| {
                                c.selected = c.index == cell.index;
                                c
                            })
                            .collect::<Vec<Cell>>();

                        self.selected_cell = Some(*cell);
                    }
                    Task::none()
                }
            },
            Message::WindowSize(size) => {
                println!("WindowSize: {:?}", size);

                self.size = Some(size);

                let sub_height = (size.height / 4.0) / 2.0;

                let new_keypoints = self.keypoints.clone();

                self.keypoints = new_keypoints
                    .into_iter()
                    .map(|mut c| {
                        c.height = sub_height;
                        c
                    })
                    .collect::<Vec<Cell>>();
                Task::none()
            }
            Message::WindowScale(scale) => {
                println!("Scale: {:?}", scale);
                self.scale = Some(scale);
                Task::none()
            }
            Message::Quit => window::get_latest().and_then(window::close),
        }
    }

    fn view(&self) -> Element<Message> {
        match self.size {
            Some(Size { width: _, height }) => {
                let cells: Vec<Element<Message>> = self
                    .keypoints
                    .clone()
                    .into_iter()
                    .map(cell_factory)
                    .collect();
                let grid = Grid::from_vec(cells).columns(9).height(height);

                container(grid)
                    .padding(2)
                    .center_x(Fill)
                    .center_y(Fill)
                    .into()
            }
            None => container(text("Init")).into(),
        }
    }

    fn style(&self, _theme: &iced::Theme) -> Style {
        Style {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
        }
    }

    fn window() -> window::Settings {
        window::Settings {
            transparent: true,
            decorations: false,
            resizable: false,
            fullscreen: true,
            level: window::Level::AlwaysOnTop,
            ..window::Settings::default()
        }
    }

    pub fn keyboard_subscription(&self) -> Subscription<Message> {
        const NO_MODIFIERS: Modifiers = Modifiers::empty();
        iced::event::listen_with(|event, _, _| match event {
            Keyboard(Event::KeyPressed { key, modifiers, .. }) => match modifiers {
                NO_MODIFIERS => match key {
                    Key::Named(Named::Escape) => Some(Message::Quit),
                    Key::Character(ch) => {
                        Some(Message::KeyPressed(ch.to_string().chars().next().unwrap()))
                    }
                    _ => None,
                },
                _ => match key {
                    Key::Named(Named::Escape) => Some(Message::Quit),
                    Key::Character(ch) => Some(Message::KeyPressed(
                        ch.to_string().to_uppercase().chars().next().unwrap(),
                    )),
                    _ => None,
                },
            },
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(char),
    WindowSize(Size),
    WindowScale(f32),
    Quit,
}
