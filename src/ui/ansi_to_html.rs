use vte::{Parser, Perform, Params};
use std::collections::HashMap;

// Represents a character cell in the terminal
#[derive(Clone, Default)]
struct Cell {
    char: char,
    fg_color: Option<String>,
    bg_color: Option<String>,
    bold: bool,
    italic: bool,
    underline: bool,
}

// Represents the terminal screen state
struct Screen {
    cells: HashMap<(usize, usize), Cell>,
    cursor_x: usize,
    cursor_y: usize,
    width: usize,
    height: usize,
    current_fg_color: Option<String>,
    current_bg_color: Option<String>,
    current_bold: bool,
    current_italic: bool,
    current_underline: bool,
}

impl Screen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            width,
            height,
            current_fg_color: None,
            current_bg_color: None,
            current_bold: false,
            current_italic: false,
            current_underline: false,
        }
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_x >= self.width {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        self.cells.insert((self.cursor_x, self.cursor_y), Cell {
            char: c,
            fg_color: self.current_fg_color.clone(),
            bg_color: self.current_bg_color.clone(),
            bold: self.current_bold,
            italic: self.current_italic,
            underline: self.current_underline,
        });

        self.cursor_x += 1;
    }

    fn newline(&mut self) {
        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    fn carriage_return(&mut self) {
        self.cursor_x = 0;
    }

    fn clear_screen(&mut self) {
        self.cells.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(self.width - 1);
        self.cursor_y = y.min(self.height - 1);
    }

    fn to_html(&self) -> String {
        let mut html = String::from("<pre class=\"terminal\">");
        let mut last_style = Cell::default();

        for y in 0..=self.height {
            for x in 0..self.width {
                if let Some(cell) = self.cells.get(&(x, y)) {
                    // Close previous style if needed
                    if cell.fg_color != last_style.fg_color 
                        || cell.bg_color != last_style.bg_color
                        || cell.bold != last_style.bold
                        || cell.italic != last_style.italic
                        || cell.underline != last_style.underline {
                        if last_style.fg_color.is_some() 
                            || last_style.bg_color.is_some()
                            || last_style.bold
                            || last_style.italic
                            || last_style.underline {
                            html.push_str("</span>");
                        }
                    }

                    // Open new style if needed
                    if cell.fg_color != last_style.fg_color 
                        || cell.bg_color != last_style.bg_color
                        || cell.bold != last_style.bold
                        || cell.italic != last_style.italic
                        || cell.underline != last_style.underline {
                        let mut style = Vec::new();
                        if let Some(fg) = &cell.fg_color {
                            style.push(format!("color: {}", fg));
                        }
                        if let Some(bg) = &cell.bg_color {
                            style.push(format!("background-color: {}", bg));
                        }
                        if cell.bold {
                            style.push("font-weight: bold".to_string());
                        }
                        if cell.italic {
                            style.push("font-style: italic".to_string());
                        }
                        if cell.underline {
                            style.push("text-decoration: underline".to_string());
                        }
                        if !style.is_empty() {
                            html.push_str(&format!("<span style=\"{}\">", style.join("; ")));
                        }
                        last_style = cell.clone();
                    }

                    match cell.char {
                        '<' => html.push_str("&lt;"),
                        '>' => html.push_str("&gt;"),
                        '&' => html.push_str("&amp;"),
                        c => html.push(c),
                    }
                } else {
                    html.push(' ');
                }
            }
            html.push('\n');
        }

        if last_style.fg_color.is_some() 
            || last_style.bg_color.is_some()
            || last_style.bold
            || last_style.italic
            || last_style.underline {
            html.push_str("</span>");
        }

        html.push_str("</pre>");
        html
    }
}

impl Perform for Screen {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => self.carriage_return(),
            b'\n' => self.newline(),
            _ => {},
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'm' => {
                // Handle SGR (Select Graphic Rendition) parameters
                for param in params.iter() {
                    match param[0] {
                        0 => {
                            // Reset all attributes
                            self.current_fg_color = None;
                            self.current_bg_color = None;
                            self.current_bold = false;
                            self.current_italic = false;
                            self.current_underline = false;
                        }
                        1 => self.current_bold = true,
                        3 => self.current_italic = true,
                        4 => self.current_underline = true,
                        30..=37 => {
                            self.current_fg_color = Some(match param[0] {
                                30 => "#000000",
                                31 => "#cd0000",
                                32 => "#00cd00",
                                33 => "#cdcd00",
                                34 => "#0000ee",
                                35 => "#cd00cd",
                                36 => "#00cdcd",
                                37 => "#e5e5e5",
                                _ => unreachable!(),
                            }.to_string());
                        }
                        40..=47 => {
                            self.current_bg_color = Some(match param[0] {
                                40 => "#000000",
                                41 => "#cd0000",
                                42 => "#00cd00",
                                43 => "#cdcd00",
                                44 => "#0000ee",
                                45 => "#cd00cd",
                                46 => "#00cdcd",
                                47 => "#e5e5e5",
                                _ => unreachable!(),
                            }.to_string());
                        }
                        _ => {},
                    }
                }
            }
            'H' | 'f' => {
                // Cursor Position
                let mut iter = params.iter();
                let y = iter.next().map(|p| p[0]).unwrap_or(1) - 1;
                let x = iter.next().map(|p| p[0]).unwrap_or(1) - 1;
                self.move_cursor(x as usize, y as usize);
            }
            'A' => {
                // Cursor Up
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_y = self.cursor_y.saturating_sub(n as usize);
            }
            'B' => {
                // Cursor Down
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_y = (self.cursor_y + n as usize).min(self.height - 1);
            }
            'C' => {
                // Cursor Forward
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_x = (self.cursor_x + n as usize).min(self.width - 1);
            }
            'D' => {
                // Cursor Back
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_x = self.cursor_x.saturating_sub(n as usize);
            }
            'J' => {
                // Clear screen
                match params.iter().next().map(|p| p[0]).unwrap_or(0) {
                    2 => self.clear_screen(),
                    _ => {},
                }
            }
            _ => {},
        }
    }
}

pub fn ansi_to_html(text: &str) -> String {
    let mut parser = Parser::new();
    let mut screen = Screen::new(132, 1000); // Reasonable terminal size

    for byte in text.bytes() {
        parser.advance(&mut screen, &[byte]);
    }

    screen.to_html()
}
