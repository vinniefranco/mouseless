use iced::Event::Keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::{Event, Key};
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
#[derive(Debug, Clone)]
enum State {
    MainGrid,
    MainCellSelected,
    SubGridSelected,
}

impl Default for State {
    fn default() -> Self {
        Self::MainGrid
    }
}

#[derive(Default)]
struct MouseApplication {
    device: Mouse,
    keypoints: Vec<Cell>,
    selected_cell: Option<Cell>,
    sub_chars: Vec<char>,
    size: Option<Size>,
    scale: Option<f32>,
    state: State,
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

#[derive(Clone, Copy)]
struct Coord {
    x: f32,
    y: f32,
    x_pad: f32,
    y_pad: f32,
    width: f32,
    height: f32,
}

impl Coord {
    fn new(size: Size, index: u32) -> Self {
        let Size { width, height } = size;
        let row = index / 9;
        let col = index % 9;
        let cell_width = width / 9.0;
        let cell_height = height / 4.0;
        let x = cell_width * col as f32 / 2.0;
        let y = cell_height * row as f32 / 2.0;
        let x_pad = cell_width / 2.0 / 2.0;
        let y_pad = cell_height / 2.0 / 2.0;

        println!("cell={row}:{col} cell_pos={x}x{y} ");

        Self {
            x,
            y,
            x_pad,
            y_pad,
            width: cell_width,
            height: cell_height,
        }
    }

    fn center_x(self) -> i32 {
        (self.x + self.x_pad) as i32
    }

    fn center_y(self) -> i32 {
        (self.y + self.y_pad) as i32
    }

    fn xy_for_sub_index(size: Size, cell_idx: u32, sub_cell_idx: usize) -> (i32, i32) {
        let coordinates = Coord::new(size, cell_idx);
        let sub_row = sub_cell_idx / 9;
        let sub_col = sub_cell_idx % 9;
        let sub_cell_width = coordinates.width / 9.0;
        let sub_cell_height = coordinates.height / 6.0;
        let x = sub_cell_width * sub_col as f32 / 2.0 + coordinates.x;
        let y = sub_cell_height * sub_row as f32 / 2.0 + coordinates.y;
        let x_pad = (sub_cell_width / 2.0 / 2.0) as i32;
        let y_pad = (sub_cell_height / 2.0 / 2.0) as i32;

        (x as i32 + x_pad, y as i32 + y_pad)
    }
}

impl Cell {
    fn new(first_char: char, sub_select: char, index: u32, label: &'static str) -> Self {
        Self {
            height: 0.0,
            selected: false,
            sub_cells: false,
            first_char,
            index,
            label,
            sub_select,
        }
    }

    fn coordinates_for_screen_size(self, size: Size) -> Coord {
        Coord::new(size, self.index)
    }

    fn set_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    fn set_selected(mut self, idx: u32) -> Self {
        self.selected = self.index == idx;
        self
    }

    fn set_subcells(mut self, idx: u32) -> Self {
        self.sub_cells = self.index == idx;
        self
    }
}

fn cell_factory(cell: Cell) -> Element<'static, Message> {
    let contents = if cell.sub_cells {
        let sub_cells = (b'A'..=b'R')
            .chain(b'a'..=b'z')
            .chain(b'0'..=b'9')
            .map(|point| sub_cell_factory(point as char))
            .collect();

        container(
            Grid::from_vec(sub_cells)
                .columns(9)
                .height(cell.height * 2.0)
                .spacing(1),
        )
    } else {
        container(text(cell.label).size(54).font(Font::MONOSPACE)).center(Fill)
    };

    container(contents)
        .style(move |theme| container_style(theme, cell))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn sub_cell_factory(ch: char) -> Element<'static, Message> {
    container(text(ch).size(12).font(Font::MONOSPACE).center())
        .center(Fill)
        .style(move |theme: &Theme| {
            let palette = theme.palette();
            container::Style {
                border: Border {
                    color: palette.danger,
                    width: 1.0,
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
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
        // QWERTY n-grams-ish
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
                state: State::MainGrid,
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
            Message::KeyPressed(ch) => match self.state {
                State::MainGrid => {
                    if let Some(cell) = self.keypoints.clone().iter().find(|k| k.first_char == ch) {
                        let new_keypoints = self.keypoints.clone();

                        self.keypoints = new_keypoints
                            .into_iter()
                            .map(|c| c.set_selected(cell.index))
                            .collect::<Vec<Cell>>();
                        self.selected_cell = Some(*cell);
                        self.state = State::MainCellSelected;
                    }
                    Task::none()
                }
                State::MainCellSelected => {
                    let mut selected = self.selected_cell.unwrap();
                    if ch == selected.sub_select {
                        let new_keypoints = self.keypoints.clone();
                        self.keypoints = new_keypoints
                            .into_iter()
                            .map(|c| c.set_subcells(selected.index))
                            .collect::<Vec<Cell>>();
                        selected.sub_cells = true;
                        self.selected_cell = Some(selected);
                        self.state = State::SubGridSelected;
                    } else if ch == selected.first_char {
                        let coords = selected.coordinates_for_screen_size(self.size.unwrap());
                        let _ = self.device.move_to(coords.center_x(), coords.center_y());
                        return window::get_latest().and_then(window::close);
                    }

                    Task::none()
                }
                State::SubGridSelected => {
                    let selected = self.selected_cell.unwrap();

                    if let Some(sub_cell_idx) = self.sub_chars.iter().position(|&c| c == ch) {
                        let (x, y) = Coord::xy_for_sub_index(
                            self.size.unwrap(),
                            selected.index,
                            sub_cell_idx,
                        );
                        let _ = self.device.move_to(x, y);
                        return window::get_latest().and_then(window::close);
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
                    .map(|c| c.set_height(sub_height))
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
                // Build out the main cells
                let cells: Vec<Element<Message>> = self
                    .keypoints
                    .clone()
                    .into_iter()
                    .map(cell_factory)
                    .collect();
                // Setup grid
                let grid = Grid::from_vec(cells).columns(9).height(height);

                // Place it.
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
            background_color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            text_color: Color::WHITE,
        }
    }

    fn window() -> window::Settings {
        window::Settings {
            decorations: false,
            fullscreen: true,
            resizable: false,
            transparent: true,
            level: window::Level::AlwaysOnTop,
            ..window::Settings::default()
        }
    }

    pub fn keyboard_subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _, _| match event {
            Keyboard(Event::KeyPressed { key, modifiers, .. }) => match key {
                Key::Named(Named::Escape) => Some(Message::Quit),
                Key::Character(ch) => {
                    let mut ch = ch.to_string();
                    if modifiers.shift() {
                        ch = ch.to_uppercase();
                    }
                    Some(Message::KeyPressed(ch.chars().next().unwrap()))
                }
                _ => None,
            },
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(char),
    Quit,
    WindowScale(f32),
    WindowSize(Size),
}
