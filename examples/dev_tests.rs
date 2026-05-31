use gtk::{Application, ApplicationWindow, glib, prelude::*};

use sofkit::prelude::box_wrapper::{hbox, vbox};
use sofkit::prelude::button::ReactiveButton;
use sofkit::prelude::reactive_widget::ReactiveWidget;
use sofkit::prelude::*;
use sofkit::runtime::Runtime;
use sofkit::state::{ReadState, WriteState};

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Sub,
    None,
    Mul,
    Div,
    Mod,
}

fn row<S: WriteState<f64> + 'static>(view: &S, from: i32, to: i32) -> BoxWrapper {
    hbox().children(|| {
        for i in from..to {
            button()
                .hexpand(true)
                .vexpand(true)
                .label(i.to_string())
                .on_click({
                    let view = view.clone();
                    move || {
                        view.edit(move |value| {
                            *value = format!("{value}{i}").parse().unwrap_or(0.0)
                        })
                    }
                });
        }
    })
}

fn build_ui() {
    statefull(|holder| {
        let view = holder.state(0.0);
        let store = holder.state(0.0);
        let operation = holder.state(Operation::None);

        let operation_button = {
            let view = view.clone();
            let store = store.clone();
            let operation = operation.clone();

            move |label: &'static str, op: Operation| {
                button().label(label).hexpand(true).vexpand(true).on_click({
                    let view = view.clone();
                    let store = store.clone();
                    let operation_state = operation.clone();

                    move || {
                        view.with(|value| store.replace(*value));
                        view.replace(0.0);
                        operation_state.replace(op);
                    }
                });
            }
        };
        vbox()
            .children(|| {
                label(view.clone());

                row(&view, 7, 10).children(|| operation_button("*", Operation::Mul));
                row(&view, 4, 7).children(|| operation_button("/", Operation::Div));
                row(&view, 1, 4).children(|| operation_button("%", Operation::Mod));

                row(&view, 0, 1).children(|| {
                    operation_button("+", Operation::Add);
                    operation_button("-", Operation::Sub);

                    button()
                        .label("=")
                        .hexpand(true)
                        .vexpand(true)
                        .on_click(move || {
                            let a = store.get().unwrap();
                            let b = view.get().unwrap();
                            let result = match operation.get().unwrap() {
                                Operation::Add => a + b,
                                Operation::Sub => a - b,
                                Operation::Mul => a * b,
                                Operation::Div => a / b,
                                Operation::Mod => a % b,
                                Operation::None => a,
                            };
                            view.replace(result);
                        });
                });
            })
            .build()
    });
}

fn build_window(app: &Application) {
    let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    Runtime::get().run_with_scope(BoxWrapper(gtk_box.clone()), || build_ui());
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Sofkit Dev Tests")
        .child(&gtk_box)
        .build();

    window.present();
}

#[tokio::main]
async fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.gtk_rs.SofkitDevTests")
        .build();

    app.connect_activate(build_window);
    app.run()
}
