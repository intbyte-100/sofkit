use crate::state::ReadState;
use crate::state::WriteState;

mod prelude;
mod state;

pub mod async_state;
mod batching;
mod scheduler;

use crate::prelude::*;
use crate::state::State;
use gtk::{Application, ApplicationWindow, glib, prelude::*};

fn build_ui() -> impl IsA<gtk::Widget> {
    statefull(|holder| {
        let counter = holder.state(0);

        let text_state = holder.state(String::new());

        let async_counter = counter.async_write();
        
        tokio::spawn(async move {
           loop {
               async_counter.replace(0).await;
               tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
           }
        });

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
                .with_state(&counter, |it, value| it.set_vexpand(value.get() % 2 == 0))
                .on_click(move || { counter.edit(|it| *it += 1) }),
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

#[tokio::main]
async fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.gtk_rs.HelloWorld1")
        .build();

    app.connect_activate(build_window);
    app.run()
}
