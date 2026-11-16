use serde_json::Value;

use crate::{
    render::Rendered,
    util::{render_template, string_prop},
};

const STYLES: &str = r#"
:host {
  display: flex;
}
"#;

fn band_color(code: i32) -> &'static str {
    match code {
        -2 => "#C3C7C0",
        -1 => "#F1D863",
        0 => "#000000",
        1 => "#8F4814",
        2 => "#FB0000",
        3 => "#FC9700",
        4 => "#FCF800",
        5 => "#00B800",
        6 => "#0000FF",
        7 => "#A803D6",
        8 => "#808080",
        _ => "#FCFCFC",
    }
}

fn break_value(value: f64) -> (i32, i32) {
    if value == 0.0 {
        return (0, 0);
    }

    let exponent = if value >= 1e10 {
        9
    } else if value >= 1e9 {
        8
    } else if value >= 1e8 {
        7
    } else if value >= 1e7 {
        6
    } else if value >= 1e6 {
        5
    } else if value >= 1e5 {
        4
    } else if value >= 1e4 {
        3
    } else if value >= 1e3 {
        2
    } else if value >= 1e2 {
        1
    } else if value >= 1e1 {
        0
    } else if value >= 1.0 {
        -1
    } else {
        -2
    };
    let base = (value / 10_f64.powi(exponent)).round() as i32;
    (base % 100, exponent)
}

pub fn render(state: &Value) -> Rendered {
    let resistance = string_prop(state, "value", "1000")
        .parse::<f64>()
        .unwrap_or(1000.0);
    let (base, exponent) = break_value(resistance);
    let band1 = band_color(base / 10);
    let band2 = band_color(base % 10);
    let band3 = band_color(exponent);
    let html = render_template(
        include_str!("templates/resistor.svg"),
        &[
            ("band1", band1.into()),
            ("band2", band2.into()),
            ("band3", band3.into()),
        ],
    );

    Rendered {
        html,
        styles: STYLES.into(),
    }
}
