mod prelude;
mod state;

use crate::prelude::*;
use crate::state::{State, StateHolder};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, glib};

fn build_ui() -> impl IsA<gtk::Widget> {
    statefull(|holder| {
        let counter = holder.state(0);

        vbox![
            label()
                .reactive()
                .text_state(&counter.clone().map(|it| format!("Clicked {it}"))),
            
            check_button()
                .reactive()
                .active_state(&counter.clone().map(|it| it % 2 == 0)),
            
            button()
                .label("Increment!")
                .margin_bottom(10)
                .margin_end(10)
                .margin_start(10)
                .margin_top(10)
                .reactive()
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
    let app = Application::builder()
        .application_id("org.gtk_rs.HelloWorld1")
        .build();

    app.connect_activate(build_window);
    app.run()
}
