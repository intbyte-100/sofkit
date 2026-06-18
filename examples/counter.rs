use gtk::gdk::Display;
use gtk::{Align, Application, ApplicationWindow, glib, prelude::*};

use sofkit::appdata::appdata;
use sofkit::prelude::box_wrapper::vbox;
use sofkit::prelude::button::ReactiveButton;
use sofkit::prelude::reactive_widget::ReactiveWidget;
use sofkit::prelude::*;
use sofkit::runtime::Runtime;
use sofkit::state::WriteState;

fn counter() {
    stateful(|holder| {
        vbox()
            .children(|| {
                let counter = holder.state(0);
                let inc = counter.clone();
                let dec = counter.clone();

                label(counter).css_class("text").halign(Align::Center);

                button()
                    .label("Increment")
                    .css_class("cbutton")
                    .on_click(move || {
                        inc.edit(|value| *value += 1);
                    });

                button()
                    .label("Decrement")
                    .css_class("cbutton")
                    .on_click(move || {
                        dec.edit(|value| *value -= 1);
                    });
            })
            .build()
    });
}

const CSS: &str = r#"
@define-color bg0 rgba(52, 49, 63, 0.5);
@define-color bg1 rgba(1, 1, 1, 0.2);
@define-color bg3 rgba(255, 255, 255, 0.3);
@define-color fg0 rgb(230, 230, 230);
@define-color fg1 rgb(255, 255, 255);

window {
    border-color: @fg1;
    background-color: @bg0;
    font-size: 1.1em;
    font-weight: bold;
    font-family: "Adwaita Sans";
}


.outer-box {
    background-color: transparent;
    padding: 0 16px 10px 16px;
}


.text {
    font-size: 20px;
    padding: 5px;
    margin: 10px 0 0 0px;
    background-color: transparent;
    border: 0;
    outline: 0;
    color: @fg1;
}


.cbutton {
    background-image: none;
    background-color: @bg1;
    color: @fg0;
    font-size: 18px;
    padding: 10px;
    margin: 4px;
    border-radius: 20px;
    border: solid 0.2px;
    border-color: @fg1;
    transition: color 150ms ease, background-color 150ms ease;
}

.cbutton:hover {
    background-color: @bg3;
    color: @fg1;
    border-color: @fg1;
}

.cbutton:active {
    background-color: alpha(@bg3, 0.7);
}
"#;

fn build_window(app: &Application) {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    gtk::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    Runtime::get().run_with_scope(BoxWrapper(root.clone()), || counter());

    ApplicationWindow::builder()
        .application(app)
        .title("Counter")
        .child(&root)
        .build()
        .present();
}

#[tokio::main]
async fn main() -> glib::ExitCode {
    appdata().with_data(|| {
        let app = Application::builder()
            .application_id("org.gtk_rs.Counter")
            .build();
        app.connect_activate(build_window);
        app.run()
    })
}
