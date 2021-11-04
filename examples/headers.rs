use fltk::{
    app,
    prelude::{GroupExt, WidgetExt},
    window,
};
use fltk_table::{SmartTable, TableOpts};

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = window::Window::default().with_size(800, 600);

    let mut table = SmartTable::default()
        .with_size(790, 590)
        .center_of_parent()
        .with_opts(TableOpts {
            rows: 30,
            cols: 15,
            ..Default::default()
        });

    wind.end();
    wind.show();

    for i in 0..30 {
        table.set_row_header_value(i, &(i + 100).to_string());
    }

    for i in 0..15 {
        table.set_col_header_value(i, &i.to_string());
    }

    app.run().unwrap();
}
