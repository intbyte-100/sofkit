use gtk::{Application, ApplicationWindow, Widget, glib, prelude::*};
use sofkit::prelude::button_builder::ReactiveButtonBuilder;
use sofkit::prelude::state_ext::StateHolderExt;

use sofkit::prelude::reactive_builder::ReactiveBuilder;
use sofkit::prelude::*;
use sofkit::state::{ReadState, State, WriteState};
use sofkit::{hbox, vbox};

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Sub,
    None,
}

fn row<S: WriteState<i32> + 'static>(view: &S, from: i32, to: i32) -> BoxWrapper {
    hbox![].append_all((from..to).map(|i| {
        button()
            .label(i.to_string())
            .reactive()
            .on_click({
                let view = view.clone();
                move || view.edit(move |value| *value = format!("{value}{i}").parse().unwrap_or(0))
            })
            .build()
    }))
}

fn operation_button<S: State<i32> + 'static, O: WriteState<Operation> + 'static>(
    label: &str,
    operation: Operation,
    view: &S,
    store: &S,
    operation_state: &O,
) -> impl IsA<Widget> {
    button()
        .label(label)
        .reactive()
        .on_click({
            let view = view.clone();
            let store = store.clone();
            let operation_state = operation_state.clone();

            move || {
                view.with(|value| store.replace(*value));
                view.replace(0);
                operation_state.replace(operation);
            }
        })
        .build()
}

fn build_ui() -> impl IsA<gtk::Widget> {
    statefull(|holder| {
        let view = holder.state(0);
        let store = holder.state(0);
        let operation = holder.state(Operation::None);

        vbox![
            label().reactive().text_state(&view),
            row(&view, 7, 10),
            row(&view, 4, 7),
            row(&view, 1, 4),
            hbox![
                operation_button("+", Operation::Add, &view, &store, &operation),
                operation_button("-", Operation::Sub, &view, &store, &operation),
                button()
                    .label("=")
                    .reactive()
                    .on_click(move || {
                        match operation.get().unwrap() {
                            Operation::Add => {
                                view.replace(store.with(|a| view.with(|b| a + b)).unwrap().unwrap())
                            }
                            Operation::Sub => {
                                view.replace(store.with(|a| view.with(|b| a - b)).unwrap().unwrap())
                            }
                            _ => {}
                        }
                    }),
            ]
        ]
        .build()
    })
}

fn build_window(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Sofkit Dev Tests")
        .child(&build_ui())
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
