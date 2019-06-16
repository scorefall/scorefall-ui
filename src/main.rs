#![recursion_limit="128"]

#[macro_use]
extern crate stdweb;

use stdweb::traits::*;
use stdweb::web::{
    document,
    Element,
    IEventTarget,
    window,
};

/// Update gamepad state view
fn animate() {
//    let list = document().create_element("ul").unwrap();

//    for pad in Gamepad::get_all() {
//        let item = document().create_element("li").unwrap();
//        item.append_child(&get_pad_state(&pad));
//        list.append_child(&item);
//    }

//    let state = document().query_selector("#state").unwrap().unwrap();

//    state.set_text_content("");
//    state.append_child(&list);

    // queue another animate() on the next frame
    window().request_animation_frame(|_| animate());
}

fn main() {
    stdweb::initialize();

    let svg: stdweb::web::Element = document()
        .get_element_by_id("canvas")
        .unwrap();

    animate();

    let string = include_str!("test.svg");

    js! {
        var svgns = "http://www.w3.org/2000/svg";
        var svg = document.getElementById("canvas");
        var shape = document.createElementNS(svgns, "circle");
        shape.setAttributeNS(null, "id", "thisid");
        shape.setAttributeNS(null, "cx", 25);
        shape.setAttributeNS(null, "cy", 25);
        shape.setAttributeNS(null, "r",  20);
        shape.setAttributeNS(null, "fill", "green");
        svg.appendChild(shape);

        alert(shape);

//        @{svg}.appendChild();
    };

    stdweb::event_loop();
}
