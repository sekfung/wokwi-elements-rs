use serde_json::{json, Value};

use crate::{
    pins::PinInfo,
    render::Rendered,
    util::{format_js_number, numberish_prop},
};

const PIXEL_WIDTH: f64 = 5.66;
const PIXEL_HEIGHT: f64 = 5.0;
const MM_TO_PIX: f64 = 3.78;

const PATTERN: &str = r##"<rect x=".33308" y="0" width="5" height="5" fill="#fff"></rect><rect x=".016709" y=".4279" width=".35114" height=".9" fill="#eaeaea"></rect><rect x="0" y="3.6518" width=".35114" height=".9" fill="#eaeaea"></rect><rect x="5.312" y="3.6351" width=".35114" height=".9" fill="#eaeaea"></rect><rect x="5.312" y=".3945" width=".35114" height=".9" fill="#eaeaea"></rect><circle cx="2.8331" cy="2.5" r="2.1" fill="#ddd"></circle><circle cx="2.8331" cy="2.5" r="1.7325" fill="#e6e6e6"></circle><g fill="#bfbfbf"><path d="m4.3488 3.3308s-0.0889-0.087-0.0889-0.1341c0-0.047-6e-3 -1.1533-6e-3 -1.1533s-0.0591-0.1772-0.2008-0.1772c-0.14174 0-0.81501 0.012-0.81501 0.012s-0.24805 0.024-0.23624 0.3071c0.0118 0.2835 0.032 2.0345 0.032 2.0345 0.54707-0.046 1.0487-0.3494 1.3146-0.8888z"></path><path d="m4.34 1.6405h-1.0805s-0.24325 0.019-0.26204-0.2423l6e-3 -0.6241c0.57782 0.075 1.0332 0.3696 1.3366 0.8706z"></path><path d="m2.7778 2.6103-0.17127 0.124-0.8091-0.012c-0.17122-0.019-0.17062-0.2078-0.17062-0.2078-1e-3 -0.3746 1e-3 -0.2831-9e-3 -0.8122l-0.31248-0.018s0.43453-0.9216 1.4786-0.9174c-1.1e-4 0.6144-4e-3 1.2289-6e-3 1.8434z"></path><path d="m2.7808 3.0828-0.0915-0.095h-0.96857l-0.0915 0.1447-3e-3 0.1127c0 0.065-0.12108 0.08-0.12108 0.08h-0.20909c0.55906 0.9376 1.4867 0.9155 1.4867 0.9155 1e-3 -0.3845-2e-3 -0.7692-2e-3 -1.1537z"></path></g><path d="m4.053 1.8619c-0.14174 0-0.81494 0.013-0.81494 0.013s-0.24797 0.024-0.23616 0.3084c3e-3 0.077 5e-3 0.3235 9e-3 0.5514h1.247c-2e-3 -0.33-4e-3 -0.6942-4e-3 -0.6942s-0.0593-0.1781-0.20102-0.1781z" fill="#666"></path>"##;

pub fn render(state: &Value) -> Rendered {
    let rows = numberish_prop(state, "rows", 8.0).max(0.0) as usize;
    let cols = numberish_prop(state, "cols", 8.0).max(0.0) as usize;
    let row_spacing = numberish_prop(state, "rowSpacing", 1.0);
    let col_spacing = numberish_prop(state, "colSpacing", 1.0);
    let blur_light = state
        .get("blurLight")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let pat_width = PIXEL_WIDTH + col_spacing;
    let pat_height = PIXEL_HEIGHT + row_spacing;
    let width = PIXEL_WIDTH * cols as f64 + col_spacing * (cols.saturating_sub(1)) as f64;
    let height = PIXEL_HEIGHT * rows as f64 + row_spacing * (rows.saturating_sub(1)) as f64;
    let colors = state.get("_pixelColors").and_then(Value::as_array);

    let mut pixels = String::new();
    for row in 0..rows {
        for col in 0..cols {
            let index = row * cols + col;
            let (red_opacity, green_opacity, blue_opacity, glow_opacity, glow_fill) = colors
                .and_then(|colors| colors.get(index))
                .and_then(Value::as_object)
                .map(neopixel_styles)
                .unwrap_or((0.0, 0.0, 0.0, 0.0, String::new()));
            let glow_style = if glow_fill.is_empty() {
                format!("opacity: {}", format_js_number(glow_opacity))
            } else {
                format!(
                    "opacity: {}; fill: {glow_fill}",
                    format_js_number(glow_opacity)
                )
            };
            pixels.push_str(&format!(
                r##"<g transform="translate({} {})" class="pixel"><ellipse cx="2.5" cy="2.3" rx="0.3" ry="0.3" fill="red" opacity="0" style="opacity: {}"></ellipse><ellipse cx="3.5" cy="3.2" rx="0.3" ry="0.3" fill="green" opacity="0" style="opacity: {}"></ellipse><ellipse cx="3.3" cy="1.45" rx="0.3" ry="0.3" fill="blue" opacity="0" style="opacity: {}"></ellipse><ellipse cx="3" cy="2.5" rx="2.2" ry="2.2" opacity="0" style="{glow_style}"></ellipse></g>"##,
                format_js_number(pat_width * col as f64),
                format_js_number(pat_height * row as f64),
                format_js_number(red_opacity),
                format_js_number(green_opacity),
                format_js_number(blue_opacity),
            ));
        }
    }
    let group_style = if blur_light {
        "filter: url(#blurLight)"
    } else {
        ""
    };
    let html = format!(
        r##"<svg width="{}mm" height="{}mm" version="1.1" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg"><filter id="blurLight" x="-0.8" y="-0.8" height="2.8" width="2.8"><feGaussianBlur stdDeviation="0.3"></feGaussianBlur></filter><pattern id="pixel" width="{}" height="{}" patternUnits="userSpaceOnUse">{PATTERN}</pattern><rect width="{}" height="{}" fill="url(#pixel)"></rect><g style="{group_style}">{pixels}</g></svg>"##,
        format_js_number(width),
        format_js_number(height),
        format_js_number(width),
        format_js_number(height),
        format_js_number(pat_width),
        format_js_number(pat_height),
        format_js_number(width),
        format_js_number(height),
    );

    Rendered {
        html,
        styles: ":host { display: flex; }".into(),
    }
}

pub fn pin_info(state: &Value) -> Vec<PinInfo> {
    let rows = numberish_prop(state, "rows", 8.0);
    let cols = numberish_prop(state, "cols", 8.0);
    let row_spacing = numberish_prop(state, "rowSpacing", 1.0);
    let col_spacing = numberish_prop(state, "colSpacing", 1.0);
    let p = 2.54 * MM_TO_PIX;
    let center_x = ((cols * (col_spacing + PIXEL_WIDTH)) / 2.0) * MM_TO_PIX;
    let y = rows * (row_spacing + PIXEL_HEIGHT) * MM_TO_PIX;
    vec![
        pin(
            "GND",
            center_x - 1.5 * p,
            y,
            json!({ "type": "power", "signal": "GND" }),
        ),
        pin(
            "VCC",
            center_x - 0.5 * p,
            y,
            json!({ "type": "power", "signal": "VCC" }),
        ),
        pin("DIN", center_x + 0.5 * p, y, Value::Null),
        pin("DOUT", center_x + 1.5 * p, y, Value::Null),
    ]
}

pub fn invoke(state: &mut Value, method: &str, args: &Value) -> bool {
    match method {
        "setPixel" => {
            let Some(args) = args.as_array() else {
                return false;
            };
            let row = args.first().and_then(Value::as_u64).unwrap_or(0) as usize;
            let col = args.get(1).and_then(Value::as_u64).unwrap_or(0) as usize;
            let Some(color) = args.get(2).cloned() else {
                return false;
            };
            let rows = numberish_prop(state, "rows", 8.0).max(0.0) as usize;
            let cols = numberish_prop(state, "cols", 8.0).max(0.0) as usize;
            if row >= rows || col >= cols {
                return false;
            }
            let index = row * cols + col;
            let object = state.as_object_mut().expect("component state object");
            let colors = object
                .entry("_pixelColors")
                .or_insert_with(|| Value::Array(vec![]));
            let colors = colors.as_array_mut().expect("_pixelColors array");
            while colors.len() <= index {
                colors.push(Value::Null);
            }
            colors[index] = color;
            true
        }
        "reset" => {
            if let Some(object) = state.as_object_mut() {
                object.insert("_pixelColors".into(), Value::Array(vec![]));
            }
            true
        }
        "setPixelColors" => {
            let Some(colors) = args.as_array() else {
                return false;
            };
            if let Some(object) = state.as_object_mut() {
                object.insert("_pixelColors".into(), Value::Array(colors.clone()));
            }
            true
        }
        "animationFrame" => {
            let time = args
                .as_array()
                .and_then(|values| values.first())
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let rows = numberish_prop(state, "rows", 8.0).max(0.0) as usize;
            let cols = numberish_prop(state, "cols", 8.0).max(0.0) as usize;
            let mut colors = Vec::with_capacity(rows * cols);
            for row in 0..rows {
                for col in 0..cols {
                    let radius = (((row as f64 - rows as f64 / 2.0 + 0.5).powi(2))
                        + (col as f64 - cols as f64 / 2.0 + 0.5).powi(2))
                    .sqrt();
                    colors.push(json!({
                        "r": pixel_value(radius * 100.0 + time),
                        "g": pixel_value(radius * 100.0 + time + 200.0),
                        "b": pixel_value(radius * 100.0 + time + 400.0),
                    }));
                }
            }
            if let Some(object) = state.as_object_mut() {
                object.insert("_pixelColors".into(), Value::Array(colors));
            }
            true
        }
        _ => false,
    }
}

fn pixel_value(value: f64) -> f64 {
    let value = value % 2000.0;
    if value > 1000.0 {
        1.0 - (value % 1000.0) / 1000.0
    } else {
        (value % 1000.0) / 1000.0
    }
}

fn neopixel_styles(color: &serde_json::Map<String, Value>) -> (f64, f64, f64, f64, String) {
    let red = numberish_prop(&Value::Object(color.clone()), "r", 0.0);
    let green = numberish_prop(&Value::Object(color.clone()), "g", 0.0);
    let blue = numberish_prop(&Value::Object(color.clone()), "b", 0.0);
    let spot = |value: f64| {
        if value > 0.001 {
            0.7 + value * 0.3
        } else {
            0.0
        }
    };
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let multiplier = 1.0_f64.max(2.0 - delta * 20.0);
    let glow_base = 0.1 + (max * 2.0 - delta * 5.0).max(0.0);
    let css_val = |value: f64| {
        if max == 0.0 {
            255.0
        } else {
            let glow = if value / max > 0.005 {
                0.1 + (value / max) * 0.9
            } else {
                0.0
            };
            (glow * multiplier).min(1.0).floor() * 255.0
        }
    };
    let glow_opacity = if max > 0.005 {
        glow_base + max * (1.0 - glow_base)
    } else {
        0.0
    };
    (
        spot(red),
        spot(green),
        spot(blue),
        glow_opacity,
        format!(
            "rgb({}, {}, {})",
            css_val(red),
            css_val(green),
            css_val(blue)
        ),
    )
}

fn pin(name: &str, x: f64, y: f64, signal: Value) -> PinInfo {
    PinInfo {
        name: name.into(),
        x,
        y,
        number: None,
        description: None,
        signals: if signal.is_null() {
            vec![]
        } else {
            vec![signal]
        },
        no_breadboard: None,
    }
}
