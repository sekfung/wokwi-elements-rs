use serde_json::Value;

use crate::{
    generated,
    render::Rendered,
    util::{format_js_number, number_prop},
};

pub fn render(state: &Value) -> Rendered {
    let red = number_prop(state, "r", 0.0);
    let green = number_prop(state, "g", 0.0);
    let blue = number_prop(state, "b", 0.0);
    let max_opacity = red.max(green).max(blue);
    let min_opacity = red.min(green).min(blue);
    let opacity_delta = max_opacity - min_opacity;
    let multiplier = 1.0_f64.max(2.0 - opacity_delta * 20.0);
    let glow_base = 0.1 + (max_opacity * 2.0 - opacity_delta * 5.0).max(0.0);
    let css_val = |value: f64| {
        if max_opacity == 0.0 {
            255.0
        } else {
            let glow_color = if value / max_opacity > 0.005 {
                0.1 + (value / max_opacity) * 0.9
            } else {
                0.0
            };
            ((glow_color * multiplier).min(1.0) * 255.0).floor()
        }
    };
    let css_color = format!(
        "rgb({}, {}, {})",
        css_val(red),
        css_val(green),
        css_val(blue)
    );
    let bkg_white = if max_opacity > 0.1 && opacity_delta < 0.2 {
        242.0 - (max_opacity * 50.0 * (1.0 - opacity_delta / 0.2)).floor()
    } else {
        242.0
    };
    let background = format!("rgb({0}, {0}, {0})", format_js_number(bkg_white));
    let spot_opacity = |value: f64| {
        if value > 0.001 {
            0.7 + value * 0.3
        } else {
            0.0
        }
    };
    let glow_opacity = |value: f64| {
        if value > 0.005 {
            glow_base + value * (1.0 - glow_base)
        } else {
            0.0
        }
    };

    let base = generated::render("wokwi-neopixel").expect("neopixel default render");
    let html = base
        .html
        .replace(
            r##"<feGaussianBlur stdDeviation="0.1"></feGaussianBlur>"##,
            &format!(
                r##"<feGaussianBlur stdDeviation="{}"></feGaussianBlur>"##,
                format_js_number(0.1_f64.max(max_opacity))
            ),
        )
        .replace(
            r##"fill="rgb(242, 242, 242)""##,
            &format!(r##"fill="{background}""##),
        )
        .replace(
            r##"<ellipse cx="2.5" cy="2.3" rx="0.3" ry="0.3" fill="red" filter="url(#light1)" opacity="0"></ellipse>"##,
            &format!(
                r##"<ellipse cx="2.5" cy="2.3" rx="0.3" ry="0.3" fill="red" filter="url(#light1)" opacity="{}"></ellipse>"##,
                format_js_number(spot_opacity(red))
            ),
        )
        .replace(
            r##"<ellipse cx="3.5" cy="3.2" rx="0.3" ry="0.3" fill="green" filter="url(#light1)" opacity="0"></ellipse>"##,
            &format!(
                r##"<ellipse cx="3.5" cy="3.2" rx="0.3" ry="0.3" fill="green" filter="url(#light1)" opacity="{}"></ellipse>"##,
                format_js_number(spot_opacity(green))
            ),
        )
        .replace(
            r##"<ellipse cx="3.3" cy="1.45" rx="0.3" ry="0.3" fill="blue" filter="url(#light1)" opacity="0"></ellipse>"##,
            &format!(
                r##"<ellipse cx="3.3" cy="1.45" rx="0.3" ry="0.3" fill="blue" filter="url(#light1)" opacity="{}"></ellipse>"##,
                format_js_number(spot_opacity(blue))
            ),
        )
        .replace(
            r##"<ellipse cx="3" cy="2.5" rx="2.2" ry="2.2" filter="url(#light2)" opacity="0" fill="rgb(255, 255, 255)"></ellipse>"##,
            &format!(
                r##"<ellipse cx="3" cy="2.5" rx="2.2" ry="2.2" filter="url(#light2)" opacity="{}" fill="{css_color}"></ellipse>"##,
                format_js_number(glow_opacity(max_opacity))
            ),
        );

    Rendered {
        html,
        styles: base.styles,
    }
}
