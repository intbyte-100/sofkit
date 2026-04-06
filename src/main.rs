mod prelude;
mod state;

pub mod app;

use crate::prelude::*;
use crate::state::{State, StateHolder};
use gtk::{Application, ApplicationWindow, Box, glib};
use gtk::{Button, prelude::*};

fn build_ui() -> impl IsA<gtk::Widget> {
    statefull(|holder| {
        let counter = holder.state(0);
        let text_state = holder.state(String::new());

        vbox![
            label().reactive().text_state(&counter),
            entry().reactive().bind_state_two_way(text_state.clone()),
            entry().reactive().bind_state_two_way(text_state.clone()),
            
            hbox![].append_all((0..10).map(|i| {
                check_button()
                    .margin_end(10)
                    .reactive()
                    .active_state(&counter.map(move |c| (*c + i) % 2 == 0))
                    .build()
            })),
            button()
                .margin_bottom(10)
                .margin_end(10)
                .margin_start(10)
                .margin_top(10)
                .reactive()
                .label_state(&text_state)
                .with_state(&counter, |it, value| it.set_vexpand(value % 2 == 0))
                .on_click(move || {
                    counter.edit(|it| *it += 1);
                }),
        ]
        .build()
    })
}

fn build_window(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&build_ui())
        .build();

    window.present();
}

fn main() -> glib::ExitCode {
    app::init();

    let app = Application::builder()
        .application_id("org.gtk_rs.HelloWorld1")
        .build();

    app.connect_activate(build_window);
    app.run()
}
