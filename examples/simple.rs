use fltk::{
    app, enums,
    prelude::{GroupExt, WidgetExt},
    window,
};
use fltk_table::{SmartTable, TableOpts};

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = window::Window::default().with_size(800, 600);

    // We pass the rows and columns thru the TableOpts field
    let mut table = SmartTable::default()
        .with_size(790, 590)
        .center_of_parent()
        .with_opts(TableOpts {
            rows: 30,
            cols: 30,
            editable: true,
            ..Default::default()
        });

    wind.end();
    wind.show();

    // set the value at the row,column 4,5 to "another", notice that indices start at 0
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
