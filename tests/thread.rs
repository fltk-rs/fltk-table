use fltk::{
    app, enums,
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

    std::thread::spawn({
        let mut table = table.clone();
        move || {
        app::sleep(0.1);
        loop {
            // Just filling the vec with some values
            for i in 0..30 {
                for j in 0..15 {
                    table.set_cell_value(i, j, &(i + j).to_string());
                    app::sleep(0.03);
                    app::awake();
                    table.redraw();
                }
            }
        }
    }});

    std::thread::spawn({
        let mut table = table.clone();
        move || {
        app::sleep(0.11);
        loop {
            // Just filling the vec with some values
            for i in 0..30 {
                for j in 0..15 {
                    table.set_cell_value(0, 0, &(i + j).to_string());
                    app::sleep(0.01);
                    app::awake();
                    table.redraw();
                }
            }
        }
    }});

    std::thread::spawn(move || {
        app::sleep(0.12);
        loop {
            // Just filling the vec with some values
            for i in 0..30 {
                for j in 0..15 {
                    table.set_cell_value(0, 0, &(i + j).to_string());
                    app::sleep(0.01);
                    app::awake();
                    table.redraw();
                }
            }
        }
    });

    // To avoid closing the window on hitting the escape key
    wind.set_callback(move |_| {
        if app::event() == enums::Event::Close {
            app.quit();
        }
    });

    app.run().unwrap();
}
