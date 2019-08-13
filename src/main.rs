#![allow(non_upper_case_globals)]
#![recursion_limit="128"]

#[macro_use]
extern crate stdweb;

macro_rules! log {
    () => (js! { console.log!("") });
    ($($arg:tt)*) => {
        let text = format!("{}", format_args!($($arg)*));
        js! { console.log(@{text}) }
    };
}

use stdweb::traits::*;
use stdweb::web::{
    document,
    Element,
    IEventTarget,
    window,
};

use score2svg::svg::node::element::path::{Command, Data};
use score2svg::svg::node::element::tag::{Use, Type, Path};
use score2svg::svg::parser::Event;

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

    let mut program = scorefall::Program::new();

    let svg: stdweb::web::Element = document()
        .get_element_by_id("canvas")
        .unwrap();

    let content: stdweb::web::Element = document()
        .get_element_by_id("content")
        .unwrap();

    let scaledown = 25_000.0;

    js! {
        window.onresize = function() {
            var content = document.getElementById("content");
            var ratio = content.clientHeight / content.clientWidth;
            var svg = document.getElementById("canvas");
            var viewbox = "0 0 " + @{scaledown} + " " + (@{scaledown} * ratio);
            svg.setAttributeNS(null, "viewBox", viewbox);
        };
    }

    let w: f64 = if let stdweb::Value::Number(n) = js! {
        return @{&content}.clientWidth;
    } {
        n.into()
    } else {
        panic!("Failed to get width");
    };

    let h: f64 = if let stdweb::Value::Number(n) = js! {
        return @{&content}.clientHeight;
    } {
        n.into()
    } else {
        panic!("Failed to get height");
    };

    animate();

    log!("YA");

    const SVGNS: &'static str = "http://www.w3.org/2000/svg";
    let sc = &program.scof;
    let string = score2svg::test_svg(score2svg::DEFAULT, scaledown as i32, sc);
    let doc = score2svg::svg::read(std::io::Cursor::new(string)).unwrap();

    let ratio = h / w;
    let viewbox = format!("0 0 {} {}", scaledown, scaledown * ratio);
    let svg = js! {
        var svg = document.getElementById("canvas");
        svg.setAttributeNS(null, "viewBox", @{viewbox});
        return svg;
    };

    let defs = js! {
        var defs = document.createElementNS(@{SVGNS}, "defs");
        return defs;
    };

    let page = js! {
        var page = document.createElementNS(@{SVGNS}, "g");
        return page;
    };

    let mut is_defs = false;

    for event in doc {
        match event {
            Event::Tag(Path, _, attributes) => {
                log!("Adding path");
                let data = attributes.get("d").unwrap().to_string();
                let shape = js! {
                    var shape = document.createElementNS(@{SVGNS}, "path");
                    shape.setAttributeNS(null, "d", @{data});
                    return shape;
                };
                if let Some(fill) = attributes.get("fill") {
                    let fill = fill.to_string();

                    js! {
                        @{&shape}.setAttributeNS(null, "fill", @{fill});
                    }
                }
                if is_defs {
                    let id = attributes.get("id").unwrap().to_string();
                    js! {
                        @{&shape}.setAttributeNS(null, "id", @{id});
                        @{&defs}.appendChild(@{&shape});
                    }
                } else {
                    js! {
                        @{&page}.appendChild(@{&shape});
                    }
                }
            }
            Event::Tag(Use, _, attributes) => {
                let x = attributes.get("x").unwrap().to_string();
                let y = attributes.get("y").unwrap().to_string();
                let xlink = attributes.get("xlink:href").unwrap().to_string();

                js! {
                    var stamp = document.createElementNS(@{SVGNS}, "use");
                    stamp.setAttributeNS(null, "x", @{x});
                    stamp.setAttributeNS(null, "y", @{y});
                    stamp.setAttributeNS(null, "href", @{xlink});
                    @{&page}.appendChild(stamp);
                }
            }
            Event::Tag("defs", Type::Start, _) => {
                log!("DEFS = TRUE");
                is_defs = true;
            }
            Event::Tag("defs", Type::End, _) => {
                log!("DEFS = FALSE");
                is_defs = false;
                js! {
                    @{&svg}.appendChild(@{&defs});
                }
            }
            _ => {}
        }
    }

    js! {
        @{&svg}.appendChild(@{page});
    };

    stdweb::event_loop();
}
