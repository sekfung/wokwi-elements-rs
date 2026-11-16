use serde_json::Value;

use crate::{
    generated,
    render::Rendered,
    util::{format_js_number, number_prop},
};

pub fn render(state: &Value) -> Rendered {
    let led_red = number_prop(state, "ledRed", 0.0);
    let led_green = number_prop(state, "ledGreen", 0.0);
    let led_blue = number_prop(state, "ledBlue", 0.0);
    let brightness = led_red.max(led_green).max(led_blue);
    let opacity = if brightness != 0.0 {
        0.2 + brightness * 0.6
    } else {
        0.0
    };
    let base = generated::render("wokwi-rgb-led").expect("rgb led default render");
    let html = base
        .html
        .replace(
            r##"<filter id="ledFilterRed" x="-0.8" y="-0.8" height="5.2" width="5.8">
          <feGaussianBlur stdDeviation="0"></feGaussianBlur>"##,
            &format!(
                r##"<filter id="ledFilterRed" x="-0.8" y="-0.8" height="5.2" width="5.8"><feGaussianBlur stdDeviation="{}"></feGaussianBlur>"##,
                format_js_number(led_red * 3.0)
            ),
        )
        .replace(
            r##"<filter id="ledFilterGreen" x="-0.8" y="-0.8" height="5.2" width="5.8">
          <feGaussianBlur stdDeviation="0"></feGaussianBlur>"##,
            &format!(
                r##"<filter id="ledFilterGreen" x="-0.8" y="-0.8" height="5.2" width="5.8"><feGaussianBlur stdDeviation="{}"></feGaussianBlur>"##,
                format_js_number(led_green * 3.0)
            ),
        )
        .replace(
            r##"<filter id="ledFilterBlue" x="-0.8" y="-0.8" height="5.2" width="5.8">
          <feGaussianBlur stdDeviation="0"></feGaussianBlur>"##,
            &format!(
                r##"<filter id="ledFilterBlue" x="-0.8" y="-0.8" height="5.2" width="5.8"><feGaussianBlur stdDeviation="{}"></feGaussianBlur>"##,
                format_js_number(led_blue * 3.0)
            ),
        )
        .replace(
            r##"<circle cx="1.7" cy="3" fill="rgb(255, 0, 0)" filter="url(#ledFilterRed)" r="2" opacity="0"></circle>"##,
            &format!(
                r##"<circle cx="1.7" cy="3" fill="rgb(255, 0, 0)" filter="url(#ledFilterRed)" r="{}" opacity="{}"></circle>"##,
                format_js_number(led_red * 5.0 + 2.0),
                format_js_number((led_red * 20.0).min(0.3))
            ),
        )
        .replace(
            r##"<circle cx="2.7" cy="5" fill="rgb(0, 255, 0)" filter="url(#ledFilterGreen)" r="2" opacity="0"></circle>"##,
            &format!(
                r##"<circle cx="2.7" cy="5" fill="rgb(0, 255, 0)" filter="url(#ledFilterGreen)" r="{}" opacity="{}"></circle>"##,
                format_js_number(led_green * 5.0 + 2.0),
                format_js_number((led_green * 20.0).min(0.3))
            ),
        )
        .replace(
            r##"<circle cx="0.7" cy="5" fill="rgb(1,85,253)" filter="url(#ledFilterBlue)" r="2" opacity="0"></circle>"##,
            &format!(
                r##"<circle cx="0.7" cy="5" fill="rgb(1,85,253)" filter="url(#ledFilterBlue)" r="{}" opacity="{}"></circle>"##,
                format_js_number(led_blue * 5.0 + 2.0),
                format_js_number((led_blue * 20.0).min(0.3))
            ),
        )
        .replace(
            r##"<circle cx="1.7" cy="4" r="10" filter="url(#ledFilter)" fill="rgb(0, 0, 0)" opacity="0"></circle>"##,
            &format!(
                r##"<circle cx="1.7" cy="4" r="10" filter="url(#ledFilter)" fill="rgb({}, {}, {})" opacity="{}"></circle>"##,
                format_js_number(led_red * 255.0),
                format_js_number(led_green * 255.0 + led_blue * 90.0),
                format_js_number(led_blue * 255.0),
                format_js_number(opacity)
            ),
        )
        .replace(
            r##"<circle cx="1.7" cy="4" r="13" stroke="#666" stroke-width="1" fill="none" filter="url(#ledFilter)" opacity="0"></circle>"##,
            &format!(
                r##"<circle cx="1.7" cy="4" r="13" stroke="#666" stroke-width="1" fill="none" filter="url(#ledFilter)" opacity="{}"></circle>"##,
                format_js_number(opacity)
            ),
        );

    Rendered {
        html,
        styles: base.styles,
    }
}
