/*!
 # fltk-table

A smart table widget for fltk-rs. It aims to reduce the amount of boilerplate required to create a table.

## Usage
```toml,ignored
[dependencies]
fltk = "1.2"
fltk-table = "0.1"
```

## Example
```rust,no_run
use fltk::{
    app, enums,
    prelude::{GroupExt, WidgetExt},
    window,
};
use fltk_table::{SmartTable, TableOpts};

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = window::Window::default().with_size(800, 600);

    let mut table = SmartTable::default(TableOpts {
        rows: 30,
        cols: 15,
        ..Default::default()
    })
    .with_size(790, 590)
    .center_of_parent();

    // the default is false
    table.editable(true);

    wind.end();
    wind.show();

    // set the value at the row,column 4,5 to "another"
    table.set_cell_value(3, 4, "another");

    assert_eq!(table.cell_value(3, 4), "another");

    // To avoid closing the window on hitting the escape key
    wind.set_callback(move |_| {
        if app::event() == enums::Event::Close {
            app.quit();
        }
    });

    app.run().unwrap();
}
```
You can retrieve a copy of the data using the `SmartTable::data()` method.
The TableOpts struct also takes styling elements for cells and headers:
```rust,no_run
use fltk::{enums::*, prelude::*, *};
use fltk_table::{SmartTable, TableOpts};
let mut table = SmartTable::default(TableOpts {
        rows: 30,
        cols: 15,
        cell_selection_color: Color::Red.inactive(),
        header_frame: FrameType::FlatBox,
        header_color: Color::BackGround.lighter(),
        cell_border_color: Color::White,
        ..Default::default()
    });
```
*/

#![allow(clippy::needless_doctest_main)]

use fltk::{
    app, draw,
    enums::*,
    input,
    prelude::{GroupExt, InputExt, TableExt, WidgetBase, WidgetExt},
    table,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

type StringMatrix = Vec<Vec<String>>;

// Needed to store cell information during the draw_cell call
#[derive(Default)]
struct CellData {
    pub row: i32, // row
    pub col: i32, // column
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl CellData {
    fn select(&mut self, row: i32, col: i32, x: i32, y: i32, w: i32, h: i32) {
        self.row = row;
        self.col = col;
        self.x = x;
        self.y = y;
        self.w = w;
        self.h = h;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TableOpts {
    pub rows: i32,
    pub cols: i32,
    pub cell_color: Color,
    pub cell_font: Font,
    pub cell_font_color: Color,
    pub cell_font_size: i32,
    pub cell_selection_color: Color,
    pub cell_align: Align,
    pub cell_border_color: Color,
    pub header_font: Font,
    pub header_frame: FrameType,
    pub header_color: Color,
    pub header_font_color: Color,
    pub header_font_size: i32,
    pub header_align: Align,
}

impl Default for TableOpts {
    fn default() -> Self {
        Self {
            rows: 1,
            cols: 1,
            cell_color: Color::BackGround2,
            cell_font: Font::Helvetica,
            cell_font_color: Color::Gray0,
            cell_font_size: 14,
            cell_selection_color: Color::from_u32(0x00D3_D3D3),
            cell_align: Align::Center,
            cell_border_color: Color::Gray0,
            header_font: Font::Helvetica,
            header_frame: FrameType::ThinUpBox,
            header_color: Color::FrameDefault,
            header_font_color: Color::Black,
            header_font_size: 14,
            header_align: Align::Center,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SmartTable {
    table: table::TableRow,
    data: Arc<Mutex<StringMatrix>>,
    row_headers: Arc<Mutex<Vec<String>>>,
    col_headers: Arc<Mutex<Vec<String>>>,
    editable: bool,
}

impl SmartTable {
    pub fn new<S: Into<Option<&'static str>>>(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        label: S,
        opts: TableOpts,
    ) -> Self {
        let table = table::TableRow::new(x, y, w, h, label);
        table.end();
        let data = Arc::new(Mutex::new(vec![
            vec![String::new(); opts.cols as _];
            opts.rows as _
        ]));

        let mut row_headers = vec![];
        for i in 0..opts.rows {
            row_headers.push(i.to_string());
        }
        let row_headers = Arc::new(Mutex::new(row_headers));

        let mut col_headers = vec![];
        for i in 0..opts.cols {
            col_headers.push(format!("{}", (i + 65) as u8 as char));
        }
        let col_headers = Arc::new(Mutex::new(col_headers));

        let mut temp = Self {
            table,
            data,
            row_headers,
            col_headers,
            editable: false,
        };

        let len = opts.rows;
        let inner_len = opts.cols;
        let cell = Rc::from(RefCell::from(CellData::default()));
        temp.table.set_rows(len as i32);
        temp.table.set_row_header(true);
        temp.table.set_row_resize(true);
        temp.table.set_cols(inner_len as i32);
        temp.table.set_col_header(true);
        temp.table.set_col_resize(true);
        temp.table.end();

        // Called when the table is drawn then when it's redrawn due to events
        temp.table.draw_cell({
            let cell = cell.clone();
            let data = temp.data.clone();
            let row_headers = temp.row_headers.clone();
            let col_headers = temp.col_headers.clone();
            move |t, ctx, row, col, x, y, w, h| {
                let data = data.lock().unwrap();
                let row_headers = row_headers.lock().unwrap();
                let col_headers = col_headers.lock().unwrap();
                match ctx {
                    table::TableContext::StartPage => draw::set_font(Font::Helvetica, 14),
                    table::TableContext::ColHeader => {
                        Self::draw_header(&col_headers[col as usize], x, y, w, h, &opts)
                    } // Column titles
                    table::TableContext::RowHeader => {
                        Self::draw_header(&row_headers[row as usize], x, y, w, h, &opts)
                    } // Row titles
                    table::TableContext::Cell => {
                        if t.is_selected(row, col) {
                            cell.borrow_mut().select(row, col, x, y, w, h); // Captures the cell information
                        }
                        Self::draw_data(
                            &data[row as usize][col as usize].to_string(),
                            x,
                            y,
                            w,
                            h,
                            t.is_selected(row, col),
                            &opts,
                        );
                    }
                    _ => (),
                }
            }
        });

        if temp.editable {
            let mut inp = input::Input::default();
            inp.set_trigger(CallbackTrigger::EnterKey);
            inp.hide();

            inp.set_callback({
                let cell = cell.clone();
                let data = temp.data.clone();
                let mut table = temp.table.clone();
                move |i| {
                    let cell = cell.borrow();
                    data.lock().unwrap()[cell.row as usize][cell.col as usize] = i.value();
                    i.set_value("");
                    i.hide();
                    table.redraw();
                }
            });

            inp.handle(|i, ev| match ev {
                Event::KeyUp => {
                    if app::event_key() == Key::Escape {
                        i.hide();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            });

            temp.table.handle({
                let data = temp.data.clone();
                move |_, ev| match ev {
                    Event::Released => {
                        let cell = cell.borrow();
                        inp.resize(cell.x, cell.y, cell.w, cell.h);
                        inp.set_value(&data.lock().unwrap()[cell.row as usize][cell.col as usize]);
                        inp.show();
                        inp.take_focus().ok();
                        inp.redraw();
                        true
                    }
                    _ => false,
                }
            });
        }
        temp
    }

    pub fn default(opts: TableOpts) -> Self {
        Self::new(0, 0, 0, 0, None, opts)
    }

    pub fn default_fill(opts: TableOpts) -> Self {
        Self::new(0, 0, 0, 0, None, opts)
            .size_of_parent()
            .center_of_parent()
    }

    pub fn editable(&mut self, flag: bool) {
        self.editable = flag;
    }
    pub fn is_editable(&self) -> bool {
        self.editable
    }

    pub fn data(&self) -> StringMatrix {
        self.data.lock().unwrap().clone()
    }

    pub fn redraw(&mut self) {
        self.table.redraw()
    }

    fn draw_header(txt: &str, x: i32, y: i32, w: i32, h: i32, opts: &TableOpts) {
        draw::push_clip(x, y, w, h);
        draw::draw_box(opts.header_frame, x, y, w, h, opts.header_color);
        draw::set_draw_color(opts.header_font_color);
        draw::set_font(opts.header_font, opts.header_font_size);
        draw::draw_text2(txt, x, y, w, h, opts.header_align);
        draw::pop_clip();
    }

    // The selected flag sets the color of the cell to a grayish color, otherwise white
    fn draw_data(txt: &str, x: i32, y: i32, w: i32, h: i32, selected: bool, opts: &TableOpts) {
        draw::push_clip(x, y, w, h);
        let sel_col = opts.cell_selection_color;
        let bg = opts.cell_color;
        if selected {
            draw::set_draw_color(sel_col);
        } else {
            draw::set_draw_color(bg);
        }
        draw::draw_rectf(x, y, w, h);
        draw::set_draw_color(opts.cell_font_color);
        draw::set_font(opts.cell_font, opts.cell_font_size);
        draw::draw_text2(txt, x, y, w, h, opts.cell_align);
        draw::set_draw_color(opts.cell_border_color);
        draw::draw_rect(x, y, w, h);
        draw::pop_clip();
    }

    pub fn set_cell_value(&mut self, row: i32, col: i32, val: &str) {
        self.data.lock().unwrap()[row as usize][col as usize] = val.to_string();
    }

    pub fn cell_value(&self, row: i32, col: i32) -> String {
        self.data.lock().unwrap()[row as usize][col as usize].clone()
    }
    pub fn set_row_header_value(&mut self, row: i32, val: &str) {
        self.row_headers.lock().unwrap()[row as usize] = val.to_string();
    }
    pub fn set_col_header_value(&mut self, col: i32, val: &str) {
        self.col_headers.lock().unwrap()[col as usize] = val.to_string();
    }
    pub fn row_header_value(&mut self, row: i32) -> String {
        self.row_headers.lock().unwrap()[row as usize].clone()
    }
    pub fn col_header_value(&mut self, col: i32) -> String {
        self.col_headers.lock().unwrap()[col as usize].clone()
    }
}

fltk::widget_extends!(SmartTable, table::TableRow, table);
