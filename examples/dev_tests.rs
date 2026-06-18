use gtk::gdk::Display;
use gtk::{Application, ApplicationWindow, Button, glib, prelude::*};

use sofkit::appdata::{Field, appdata, get_appdata};
use sofkit::prelude::box_wrapper::{hbox, vbox};
use sofkit::prelude::button::ReactiveButton;
use sofkit::prelude::reactive_widget::ReactiveWidget;
use sofkit::prelude::*;
use sofkit::runtime::Runtime;
use sofkit::state::ReadState;
use sofkit::value::ReactiveValue;

#[derive(Debug, Clone, Copy)]
enum CalcError {
    DivisionByZero,
    InsufficientOperands,
    InvalidExpression,
    UnknownOperator,
}

impl CalcError {
    fn message(self) -> &'static str {
        match self {
            Self::DivisionByZero => "Division by zero",
            Self::InsufficientOperands => "Insufficient operands",
            Self::InvalidExpression => "Invalid expression",
            Self::UnknownOperator => "Unknown operator",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Token {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Num(f64),
    Err(CalcError),
}

impl Token {
    fn precedence(self) -> u8 {
        match self {
            Token::Add | Token::Sub => 1,
            Token::Mul | Token::Div | Token::Mod => 2,
            _ => 0,
        }
    }

    fn display_str(self) -> String {
        match self {
            Token::Add => "+".into(),
            Token::Sub => "-".into(),
            Token::Mul => "*".into(),
            Token::Div => "/".into(),
            Token::Mod => "%".into(),
            Token::Num(v) => format!("{v}"),
            Token::Err(e) => e.message().into(),
        }
    }
}

struct Calculator {
    expr: Field<Vec<Token>>,
    result: Field<f64>,
}

impl Calculator {
    fn new() -> Self {
        Self {
            expr: Field::new(vec![Token::Num(0.0)]),
            result: Field::new(0.0),
        }
    }

    fn append(&self, token: Token) {
        self.expr.edit(|vec| {
            if matches!(vec.last(), Some(Token::Err(_))) {
                vec.clear();
            }
            let token = Self::merge_num(vec, token);
            vec.push(token);
        });
    }

    fn clear(&self) {
        self.expr.edit(|v| {
            v.clear();
            v.push(Token::Num(0.0));
        });
    }

    fn evaluate(&self) -> Result<f64, CalcError> {
        let tokens = self.expr.with(|v| v.clone());
        match Self::eval_tokens(&tokens) {
            Ok(value) => {
                self.result.edit(|r| *r = value);
                Ok(value)
            }
            Err(err) => {
                self.expr.edit(|v| {
                    v.clear();
                    v.push(Token::Err(err));
                });
                Err(err)
            }
        }
    }

    fn merge_num(vec: &mut Vec<Token>, token: Token) -> Token {
        if let Token::Num(digit) = token {
            if let Some(Token::Num(prev)) = vec.last().copied() {
                vec.pop();
                let merged = format!("{prev}{digit}").parse::<f64>().unwrap_or(digit);
                return Token::Num(merged);
            }
        }
        token
    }

    fn eval_tokens(tokens: &[Token]) -> Result<f64, CalcError> {
        let mut values: Vec<f64> = Vec::new();
        let mut ops: Vec<Token> = Vec::new();

        for &token in tokens {
            match token {
                Token::Num(x) => values.push(x),
                Token::Err(e) => return Err(e),
                op => {
                    while ops
                        .last()
                        .map_or(false, |top| top.precedence() >= op.precedence())
                    {
                        Self::apply_op(&mut values, ops.pop().unwrap())?;
                    }
                    ops.push(op);
                }
            }
        }

        while let Some(op) = ops.pop() {
            Self::apply_op(&mut values, op)?;
        }

        match values.as_slice() {
            [result] => Ok(*result),
            _ => Err(CalcError::InvalidExpression),
        }
    }

    fn apply_op(values: &mut Vec<f64>, op: Token) -> Result<(), CalcError> {
        let right = values.pop().ok_or(CalcError::InsufficientOperands)?;
        let left = values.pop().ok_or(CalcError::InsufficientOperands)?;
        let result = match op {
            Token::Add => left + right,
            Token::Sub => left - right,
            Token::Mul => left * right,
            Token::Div if right == 0.0 => return Err(CalcError::DivisionByZero),
            Token::Div => left / right,
            Token::Mod => left % right,
            _ => return Err(CalcError::UnknownOperator),
        };
        values.push(result);
        Ok(())
    }
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
    min-width: 300pt;
    min-height: 480px;
    font-size: 1.1em;
    font-weight: bold;
    font-family: "Adwaita Sans";
}


.outer-box {
    background-color: transparent;
    padding: 0 16px 10px 16px;
}


.expr {
    font-size: 20px;
    padding: 5px;
    margin: 20px 20px 20px 10px;
    background-color: transparent;
    border: 0;
    outline: 0;
    color: @fg1;
}


.calc-button {
    background-image: none;
    background-color: @bg1;
    color: @fg0;
    font-size: 18px;
    padding: 10px;
    margin: 4px;
    border-radius: 30%;
    border: solid 0.2px;
    border-color: @fg1;
    transition: color 150ms ease, background-color 150ms ease;
}

.calc-button:hover {
    background-color: @bg3;
    color: @fg1;
    border-color: @fg1;
}

.calc-button:active {
    background-color: alpha(@bg3, 0.7);
}
"#;

fn calc_button(lbl: impl ReactiveValue<String> + 'static) -> impl ReactiveButton<Button> {
    button()
        .css_class("calc-button")
        .label(lbl)
        .hexpand(true)
        .vexpand(true)
}

fn digit_row(range: std::ops::Range<i32>) -> BoxWrapper {
    hbox().children(|| {
        for i in range {
            calc_button(i.to_string()).on_click(move || {
                get_appdata::<Calculator>().append(Token::Num(i as f64));
            });
        }
    })
}

fn op_button(lbl: &'static str, op: Token) {
    calc_button(lbl).on_click(move || {
        get_appdata::<Calculator>().append(op);
    });
}

fn equals_button() {
    calc_button("=").on_click(|| {
        let calc = get_appdata::<Calculator>();
        if let Ok(value) = calc.evaluate() {
            calc.clear();
            calc.append(Token::Num(value));
        }
    });
}

fn build_ui() {
    stateful(|holder| {
        let calc = get_appdata::<Calculator>();

        let expr_view = calc.expr.make_state(holder).map(|tokens| {
            tokens
                .iter()
                .map(|t| t.display_str())
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        });

        vbox()
            .children(|| {
                label(expr_view)
                    .css_class("expr")
                    .hexpand(true)
                    .halign(gtk::Align::End);

                digit_row(7..10).children(|| op_button("*", Token::Mul));
                digit_row(4..7).children(|| op_button("/", Token::Div));
                digit_row(1..4).children(|| op_button("%", Token::Mod));
                digit_row(0..1).children(|| {
                    op_button("+", Token::Add);
                    op_button("-", Token::Sub);
                    equals_button();
                });
            })
            .build()
    });
}

fn build_window(app: &Application) {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    gtk::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    Runtime::get().run_with_scope(BoxWrapper(root.clone()), || build_ui());

    ApplicationWindow::builder()
        .application(app)
        .title("Calculator")
        .child(&root)
        .build()
        .present();
}

#[tokio::main]
async fn main() -> glib::ExitCode {
    appdata().insert(Calculator::new()).with_data(|| {
        let app = Application::builder()
            .application_id("org.gtk_rs.Calculator")
            .build();
        app.connect_activate(build_window);
        app.run()
    })
}
