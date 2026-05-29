use iced::keyboard;
use iced::widget::canvas;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Subscription, Theme};
use std::time::Instant;

const TOTAL_CHARS_PER_LINE: usize = 80;
const TOTAL_LINES: usize = 56;
const CHAR_RATIO: f32 = 2.0;
const A4_RATIO: f32 = 297.0 / 210.0;
const MAX_LAYERS: usize = 3;

const LEFT_MARGIN: usize = 6;
const RIGHT_MARGIN: usize = 6;
const TOP_MARGIN: usize = 3;
const BOTTOM_MARGIN: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineWidthMode {
    Mode68,  // текущее: 80 - 6 - 6 = 68
    Mode80,  // 80 вводимых символов
    Mode106, // 106 вводимых символов
}

impl LineWidthMode {
    fn visible_chars(&self) -> usize {
        match self {
            LineWidthMode::Mode68 => TOTAL_CHARS_PER_LINE - LEFT_MARGIN - RIGHT_MARGIN, // 68
            LineWidthMode::Mode80 => 80,
            LineWidthMode::Mode106 => 106,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            LineWidthMode::Mode68 => "68",
            LineWidthMode::Mode80 => "80",
            LineWidthMode::Mode106 => "106",
        }
    }
}

n visible_chars_per_line(mode: LineWidthMode) -> usize {
    mode.visible_chars()
}

const VISIBLE_LINES: usize = TOTAL_LINES - TOP_MARGIN - BOTTOM_MARGIN;

const ALT_DOUBLE_TAP_MS: u64 = 300;

const ALLOWED_CHARS: &str = "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\
     ABCDEFGHIJKLMNOPQRSTUVWXYZ\
     0123456789\
     .,;:!?-–—_()[]{}<>\"'`@#$%&*+=/\\|~^«»№ ";

const COLOR_BLACK: Color = Color::from_rgb(0.0, 0.0, 0.0);
const COLOR_RED: Color = Color::from_rgb(0.8, 0.1, 0.1);

fn mix_colors(colors: &[Color]) -> Color {
    if colors.is_empty() {
        return COLOR_BLACK;
    }
    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    let mut a = 0.0f32;
    for c in colors {
        r += c.r;
        g += c.g;
        b += c.b;
        a += c.a;
    }
    let count = colors.len() as f32;
    Color::from_rgba(
        (r / count).min(1.0),
        (g / count).min(1.0),
        (b / count).min(1.0),
        (a / count).min(1.0),
    )
}

fn color_with_alpha(color: Color, alpha: f32) -> Color {
    Color::from_rgba(color.r, color.g, color.b, alpha)
}

#[derive(Debug, Clone)]
struct Layer {
    ch: char,
    color: Color,
}

#[derive(Debug, Clone)]
struct Cell {
    layers: Vec<Layer>,
}

impl Cell {
    fn new() -> Self {
        Self { layers: Vec::new() }
    }

    fn push(&mut self, c: char, color: Color) {
        self.layers.push(Layer { ch: c, color });
    }

    fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    fn len(&self) -> usize {
        self.layers.len()
    }

    fn saturation_level(&self) -> usize {
        self.len().min(MAX_LAYERS)
    }
}

#[derive(Debug, Clone)]
struct TypewriterState {
    lines: Vec<Vec<Cell>>,
    cursor_col: usize,
    alt_color_inverted: bool,
    last_alt_press: Option<Instant>,
    allowed_chars: std::collections::HashSet<char>,
    line_width_mode: LineWidthMode,
    show_help: bool,
}

impl Default for TypewriterState {
    fn default() -> Self {
        let mode = LineWidthMode::Mode68;
        let visible = visible_chars_per_line(mode);
        let mut lines = Vec::with_capacity(VISIBLE_LINES);
        lines.push(create_empty_line(visible));

        let allowed_chars = ALLOWED_CHARS.chars().collect();

        Self {
            lines,
            cursor_col: 0,
            alt_color_inverted: false,
            last_alt_press: None,
            allowed_chars,
            line_width_mode: mode,
            show_help: false,
        }
    }
}

fn create_empty_line(len: usize) -> Vec<Cell> {
    (0..len).map(|_| Cell::new()).collect()
}

impl TypewriterState {
    fn visible_chars(&self) -> usize {
        visible_chars_per_line(self.line_width_mode)
    }

    fn current_line_mut(&mut self) -> Option<&mut Vec<Cell>> {
        self.lines.last_mut()
    }

    fn is_full(&self) -> bool {
        self.lines.len() > VISIBLE_LINES
            || (self.lines.len() == VISIBLE_LINES && self.cursor_col >= self.visible_chars())
    }

    fn get_color(&self, is_alt: bool) -> Color {
        match (self.alt_color_inverted, is_alt) {
            (true, true) => COLOR_BLACK,
            (true, false) => COLOR_RED,
            (false, true) => COLOR_RED,
            (false, false) => COLOR_BLACK,
        }
    }

    fn tape_color_name(&self) -> &'static str {
        if self.alt_color_inverted {
            "красный"
        } else {
            "чёрный"
        }
    }

    fn is_char_allowed(&self, c: char) -> bool {
        self.allowed_chars.contains(&c)
    }

    fn type_char(&mut self, c: char, color: Color) {
        let c = c.to_uppercase().next().unwrap_or(c);
        if !self.is_char_allowed(c) {
            return;
        }
        if self.is_full() {
            return;
        }

        let visible = self.visible_chars();
        let col = self.cursor_col;
        if col >= visible {
            return;
        }

        if let Some(line) = self.current_line_mut() {
            if col < line.len() {
                line[col].push(c, color);
                self.cursor_col = col + 1;
            }
        }

        if self.cursor_col >= visible {
            self.enter();
        }
    }

    fn enter(&mut self) {
        self.lines.push(create_empty_line(self.visible_chars()));
        self.cursor_col = 0;
        if self.lines.len() > VISIBLE_LINES {
            self.lines.remove(0);
        }
    }

    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_col < self.visible_chars() {
            self.cursor_col += 1;
        }
    }

    fn handle_alt_press(&mut self) {
        let now = Instant::now();
        if let Some(last) = self.last_alt_press {
            if (now.duration_since(last).as_millis() as u64) < ALT_DOUBLE_TAP_MS {
                self.alt_color_inverted = !self.alt_color_inverted;
                self.last_alt_press = None;
                return;
            }
        }
        self.last_alt_press = Some(now);
    }

    fn set_line_width_mode(&mut self, mode: LineWidthMode) {
        if self.line_width_mode == mode {
            return;
        }
        self.line_width_mode = mode;
        let new_visible = self.visible_chars();

        let mut new_lines = Vec::with_capacity(self.lines.len().min(VISIBLE_LINES));
        for old_line in &self.lines {
            let mut new_line = create_empty_line(new_visible);
            for (i, cell) in old_line.iter().enumerate().take(new_visible) {
                new_line[i] = cell.clone();
            }
            new_lines.push(new_line);
        }

        if new_lines.is_empty() {
            new_lines.push(create_empty_line(new_visible));
        }

        self.lines = new_lines;
        self.cursor_col = self.cursor_col.min(new_visible);
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    #[allow(dead_code)]
    fn validate(&self) -> Result<(), &'static str> {
        if self.lines.is_empty() {
            return Err("lines не должны быть пустыми");
        }
        if self.cursor_col > self.visible_chars() {
            return Err("cursor_col выходит за границы");
        }
        if self.lines.len() > VISIBLE_LINES {
            return Err("превышено максимальное количество строк");
        }
        let expected_len = self.visible_chars();
        for line in &self.lines {
            if line.len() != expected_len {
                return Err("некорректная длина строки");
            }
        }
        Ok(())
    }
}

struct TypewriterCanvas;

const HELP_TEXT: &str = r#"СПРАВКА — Пишущая машинка Consul 254

Управление:
  A-Z, А-Я, 0-9, знаки препинания  —  печать символа
  Пробел                            —  пробел
  Enter                             —  перевод строки
  Alt + Enter                       —  возврат каретки
  ← →                               —  сдвиг каретки
  Alt (двойное нажатие)             —  смена цвета ленты
  Ctrl+1 / Ctrl+2 / Ctrl+3          —  ширина строки: 68 / 80 / 106
  F1                                —  показать/скрыть эту справку

Особенности:
  • Ошибочно набранный символ остаётся на бумаге
  • Повторный удар по той же позиции перекрывает символ
  • Максимум 3 наложения на одну позицию
"#;

impl<Message> canvas::Program<Message> for TypewriterCanvas {
    type State = TypewriterState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        #[cfg(debug_assertions)]
        if let Err(e) = state.validate() {
            eprintln!("Ошибка инварианта: {}", e);
        }

        match event {
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                text,
                modifiers,
                ..
            }) => {
                let is_alt = modifiers.alt();
                let is_ctrl = modifiers.control();
                let color = state.get_color(is_alt);

                if matches!(key.as_ref(), keyboard::Key::Named(keyboard::key::Named::F1)) {
                    state.toggle_help();
                    return Some(canvas::Action::request_redraw());
                }

                if state.show_help {
                    state.show_help = false;
                    return Some(canvas::Action::request_redraw());
                }

                if is_ctrl {
                    if let keyboard::Key::Character(c) = key.as_ref() {
                        let s: &str = c.as_ref();
                        if s == "1" {
                            state.set_line_width_mode(LineWidthMode::Mode68);
                            return Some(canvas::Action::request_redraw());
                        } else if s == "2" {
                            state.set_line_width_mode(LineWidthMode::Mode80);
                            return Some(canvas::Action::request_redraw());
                        } else if s == "3" {
                            state.set_line_width_mode(LineWidthMode::Mode106);
                            return Some(canvas::Action::request_redraw());
                        }
                    }
                }

                if matches!(
                    key.as_ref(),
                    keyboard::Key::Named(keyboard::key::Named::Alt)
                ) {
                    state.handle_alt_press();
                    return Some(canvas::Action::request_redraw());
                }

                match key.as_ref() {
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        if is_alt {
                            state.carriage_return();
                        } else {
                            state.enter();
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Space) => {
                        state.type_char(' ', color);
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        state.move_left();
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        state.move_right();
                    }
                    _ => {
                        if let Some(t) = text {
                            if let Some(c) = t.chars().next() {
                                state.type_char(c, color);
                            }
                        } else if let keyboard::Key::Character(c) = key.as_ref() {
                            if let Some(ch) = c.chars().next() {
                                state.type_char(ch, color);
                            }
                        }
                    }
                }
                Some(canvas::Action::request_redraw())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let visible_chars = state.visible_chars() as f32;
        let char_w = bounds.width / (LEFT_MARGIN as f32 + visible_chars + RIGHT_MARGIN as f32);
        let char_h = char_w * CHAR_RATIO;

        // Фон листа
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            Size::new(bounds.width, bounds.height),
            Color::WHITE,
        );

        let status_line_width_chars = state.visible_chars() + 2;
        let status_line_width = status_line_width_chars as f32 * char_w;
        let status_x = (bounds.width - status_line_width) / 2.0;
        let field_top = 0.0;
        let field_bottom = TOP_MARGIN as f32 * char_h;
        let field_center_y = field_bottom / 2.0;
        let status_char_h = char_h * 0.8;
        let status_font_size = iced::Pixels(status_char_h);
        let line_gap = status_char_h * 0.3;
        let total_status_height = status_char_h * 2.0 + line_gap;
        let status_start_y = field_center_y - total_status_height / 2.0 + status_char_h / 2.0;

        let top_left_text = "КОНСУЛ 254";
        let top_right_text = format!(
            "Режим: {} символов, интервал одинарный, цвет ленты {}",
            state.line_width_mode.label(),
            state.tape_color_name()
        );

        let status_top_left = canvas::Text {
            content: top_left_text.to_string(),
            position: Point::new(status_x, status_start_y),
            color: Color::from_rgb(0.3, 0.3, 0.3),
            size: status_font_size,
            font: iced::Font::MONOSPACE,
            align_x: iced::widget::text::Alignment::Left,
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        };
        frame.fill_text(status_top_left);

        let status_top_right = canvas::Text {
            content: top_right_text,
            position: Point::new(status_x + status_line_width, status_start_y),
            color: Color::from_rgb(0.3, 0.3, 0.3),
            size: status_font_size,
            font: iced::Font::MONOSPACE,
            align_x: iced::widget::text::Alignment::Right,
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        };
        frame.fill_text(status_top_right);

        let underline_y = status_start_y + status_char_h + line_gap;
        let line_thickness = status_char_h * 0.12;
        frame.fill_rectangle(
            Point::new(status_x, underline_y - line_thickness / 2.0),
            Size::new(status_line_width, line_thickness),
            Color::from_rgb(0.3, 0.3, 0.3),
        );

        let lines_to_show = state.lines.len().min(VISIBLE_LINES);

        for display_idx in 0..lines_to_show {
            let line_idx = state.lines.len().saturating_sub(1 + display_idx);
            let Some(line) = state.lines.get(line_idx) else {
                continue;
            };

            let y = bounds.height - ((display_idx + BOTTOM_MARGIN) as f32 + 1.0) * char_h;

            for (col, cell) in line.iter().enumerate() {
                if cell.is_empty() {
                    continue;
                }

                let x = (LEFT_MARGIN + col) as f32 * char_w;
                let center_x = x + char_w / 2.0;
                let center_y = y + char_h / 2.0;

                let sat_level = cell.saturation_level();
                let count = cell.len();

                match sat_level {
                    1 => {
                        let layer = &cell.layers[0];
                        let text = canvas::Text {
                            content: layer.ch.to_string(),
                            position: Point::new(center_x, center_y),
                            color: color_with_alpha(layer.color, 0.5),
                            size: iced::Pixels(char_h * 0.8),
                            font: iced::Font::MONOSPACE,
                            align_x: iced::widget::text::Alignment::Center,
                            align_y: iced::alignment::Vertical::Center,
                            ..canvas::Text::default()
                        };
                        frame.fill_text(text);
                    }
                    2 => {
                        for (layer_idx, layer) in cell.layers.iter().enumerate() {
                            let offset_x = (layer_idx as f32 * 0.5 - 0.25) * char_w * 0.04;
                            let offset_y = (layer_idx as f32 * 0.5 - 0.25) * char_h * 0.04;
                            let text = canvas::Text {
                                content: layer.ch.to_string(),
                                position: Point::new(center_x + offset_x, center_y + offset_y),
                                color: color_with_alpha(layer.color, 0.75),
                                size: iced::Pixels(char_h * 0.8),
                                font: iced::Font::MONOSPACE,
                                align_x: iced::widget::text::Alignment::Center,
                                align_y: iced::alignment::Vertical::Center,
                                ..canvas::Text::default()
                            };
                            frame.fill_text(text);
                        }
                    }
                    _ => {
                        let mixed = mix_colors(
                            &cell
                                .layers
                                .iter()
                                .take(MAX_LAYERS)
                                .map(|l| l.color)
                                .collect::<Vec<_>>(),
                        );

                        let layers_to_show = MAX_LAYERS.min(count);
                        for layer_idx in 0..layers_to_show {
                            let layer = &cell.layers[layer_idx];
                            let offset_x = (layer_idx as f32 - 1.0) * char_w * 0.03;
                            let offset_y = (layer_idx as f32 - 1.0) * char_h * 0.03;
                            let text = canvas::Text {
                                content: layer.ch.to_string(),
                                position: Point::new(center_x + offset_x, center_y + offset_y),
                                color: mixed,
                                size: iced::Pixels(char_h * 0.8),
                                font: iced::Font::MONOSPACE,
                                align_x: iced::widget::text::Alignment::Center,
                                align_y: iced::alignment::Vertical::Center,
                                ..canvas::Text::default()
                            };
                            frame.fill_text(text);
                        }

                        for layer_idx in MAX_LAYERS..count {
                            let layer = &cell.layers[layer_idx];
                            let offset_x = ((layer_idx % 3) as f32 - 1.0) * char_w * 0.03;
                            let offset_y = ((layer_idx % 3) as f32 - 1.0) * char_h * 0.03;
                            let text = canvas::Text {
                                content: layer.ch.to_string(),
                                position: Point::new(center_x + offset_x, center_y + offset_y),
                                color: mixed,
                                size: iced::Pixels(char_h * 0.8),
                                font: iced::Font::MONOSPACE,
                                align_x: iced::widget::text::Alignment::Center,
                                align_y: iced::alignment::Vertical::Center,
                                ..canvas::Text::default()
                            };
                            frame.fill_text(text);
                        }
                    }
                }
            }
        }

        if !state.lines.is_empty() && state.cursor_col <= state.visible_chars() {
            let cursor_x = (LEFT_MARGIN + state.cursor_col) as f32 * char_w;
            let cursor_y = bounds.height - (BOTTOM_MARGIN as f32 + 1.0) * char_h;

            frame.fill_rectangle(
                Point::new(cursor_x, cursor_y),
                Size::new(char_w, char_h),
                Color::BLACK,
            );
        }

        if state.show_help {
            let help_x = 0.0;
            let help_y = TOP_MARGIN as f32 * char_h;
            let help_w = bounds.width;
            let help_h = bounds.height - (TOP_MARGIN + BOTTOM_MARGIN) as f32 * char_h;

            frame.fill_rectangle(
                Point::new(help_x, help_y),
                Size::new(help_w, help_h),
                Color::from_rgba(0.92, 0.92, 0.92, 0.97),
            );

            let help_lines: Vec<&str> = HELP_TEXT.lines().collect();
            let help_char_h = char_h * 0.75;
            let help_font_size = iced::Pixels(help_char_h * 0.75);
            let line_spacing = help_char_h * 1.15;
            // Отступ от верхнего края подложки
            let start_y = help_y + help_char_h * 1.0;
            // Отступ слева: на ширину левого поля + небольшой отступ
            let text_x = LEFT_MARGIN as f32 * char_w + char_w * 0.5;

            for (i, line) in help_lines.iter().enumerate() {
                let y = start_y + i as f32 * line_spacing;
                let text = canvas::Text {
                    content: line.to_string(),
                    position: Point::new(text_x, y),
                    color: Color::from_rgb(0.15, 0.15, 0.15),
                    size: help_font_size,
                    font: iced::Font::MONOSPACE,
                    align_x: iced::widget::text::Alignment::Left,
                    align_y: iced::alignment::Vertical::Center,
                    ..canvas::Text::default()
                };
                frame.fill_text(text);
            }
        }

        vec![frame.into_geometry()]
    }
}

fn update(_state: &mut (), _message: ()) -> iced::Task<()> {
    iced::Task::none()
}

fn view(_state: &()) -> Element<'_, ()> {
    canvas(TypewriterCanvas)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn subscription(_state: &()) -> Subscription<()> {
    Subscription::none()
}

fn main() -> iced::Result {
    iced::application(|| (), update, view)
        .title("Пишущая машинка Consul 254")
        .window(iced::window::Settings {
            size: Size::new(800.0, 800.0 * A4_RATIO),
            resizable: false,
            ..Default::default()
        })
        .subscription(subscription)
        .run()
}
