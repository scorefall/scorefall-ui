#![recursion_limit = "128"]

macro_rules! log {
    () => (js! { console.log!("") });
    ($($arg:tt)*) => {
        let text = format!("{}", format_args!($($arg)*));
        js! { console.log(@{text}) }
    };
}

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

use stdweb::js;
use stdweb::traits::*;
use stdweb::web::{
    document,
    event::{
        ContextMenuEvent, KeyDownEvent, KeyUpEvent, MouseWheelEvent,
        ResizeEvent,
    },
    window, IEventTarget,
};

use std::cell::RefCell;
use std::rc::Rc;

use score2svg::svg::node::element::tag;
use score2svg::svg::parser::Event;

use scorefall::Program;

mod input;

use input::*;

struct State {
    program: Program,
    time_old: f64,
    command: String,
    input: InputState,
    svg: stdweb::web::Element,
}

impl State {
    fn new(svg: stdweb::web::Element) -> State {
        State {
            program: Program::new(),
            time_old: 0.0,
            command: "".to_string(),
            input: InputState::new(),
            svg,
        }
    }

    fn process_input(&mut self, time: f64) {
        let _dt = (time - self.time_old) as f32;
        self.time_old = time;

        if self.input.has_input {
            if self.input.keys[KeyName::Left as usize].press() {
                println!("PRINTLN LEFT");
                log!("LEFT");
                self.program.left();
                render_score(self);
            }
            if self.input.keys[KeyName::Right as usize].press() {
                log!("RIGHT");
                self.program.right();
                render_score(self);
            }
        }

        self.input.reset();
    }

    fn run(time: f64, rc: Rc<RefCell<Self>>) {
        rc.borrow_mut().process_input(time);

        window().request_animation_frame(move |time| {
            Self::run(time, rc.clone());
        });
    }
}

/*/// Update gamepad state view
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
}*/

const SCALEDOWN: f64 = 25_000.0;

fn render_score(state: &State) {
    let svg = &state.svg;

    js! {
        @{&svg}.innerHTML = "";
    }

    let w: f64 = if let stdweb::Value::Number(n) = js! {
        return @{svg}.clientWidth;
    } {
        n.into()
    } else {
        panic!("Failed to get width");
    };

    let h: f64 = if let stdweb::Value::Number(n) = js! {
        return @{svg}.clientHeight;
    } {
        n.into()
    } else {
        panic!("Failed to get height");
    };

    const SVGNS: &str = "http://www.w3.org/2000/svg";
    let renderer = score2svg::Renderer::new(
        &state.program.scof,
        0,
        state.program.curs,
        state.program.bar,
        score2svg::DEFAULT,
        SCALEDOWN as i32,
    );
    let string = renderer.render();
    let doc = score2svg::svg::read(std::io::Cursor::new(string)).unwrap();

    let ratio = h / w;
    let viewbox = format!("0 0 {} {}", SCALEDOWN, SCALEDOWN * ratio);
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
            Event::Tag(tag::Rectangle, _, attributes) => {
                log!("Adding rect");
                let x = attributes.get("x").unwrap().to_string();
                let y = attributes.get("y").unwrap().to_string();
                let width = attributes.get("width").unwrap().to_string();
                let height = attributes.get("height").unwrap().to_string();
                let rect = js! {
                    var rect = document.createElementNS(@{SVGNS}, "rect");
                    rect.setAttributeNS(null, "x", @{x});
                    rect.setAttributeNS(null, "y", @{y});
                    rect.setAttributeNS(null, "width", @{width});
                    rect.setAttributeNS(null, "height", @{height});
                    return rect;
                };
                if let Some(fill) = attributes.get("fill") {
                    let fill = fill.to_string();

                    js! {
                        @{&rect}.setAttributeNS(null, "fill", @{fill});
                    }
                }
                js! {
                    @{&page}.appendChild(@{&rect});
                }
            },
            Event::Tag(tag::Path, _, attributes) => {
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
            Event::Tag(tag::Use, _, attributes) => {
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
            Event::Tag("defs", tag::Type::Start, _) => {
                log!("DEFS = TRUE");
                is_defs = true;
            }
            Event::Tag("defs", tag::Type::End, _) => {
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
}

fn main() {
    stdweb::initialize();

    let svg: stdweb::web::Element =
        document().get_element_by_id("canvas").unwrap();

    let state = Rc::new(RefCell::new(State::new(svg)));

    // FIXME: Use this.
    let _prompt: stdweb::web::Element =
        document().get_element_by_id("prompt").unwrap();

    window().add_event_listener(enclose!( (state) move |_: ResizeEvent| {
        let svg = &state.borrow().svg;
        js! {
            var svg = @{svg};
            var ratio = svg.clientHeight / svg.clientWidth;
            var viewbox = "0 0 " + @{SCALEDOWN} + " " + (@{SCALEDOWN} * ratio);
            svg.setAttributeNS(null, "viewBox", viewbox);
        }
    }));

    window().add_event_listener(
        enclose!( (state) move |event: ContextMenuEvent| {
        //        js! {
        //            alert("success!");
        //        }
                event.prevent_default();
            }),
    );

    // CTRL-W, CTRL-Q, CTRL-T, CTRL-N aren't picked up by this (Tested chromium,
    // firefox).
    window().add_event_listener(enclose!( (state) move |event: KeyDownEvent| {
        let is = &mut state.borrow_mut().input;
        let key = event.key();
        let code = event.code();

        if code != "F11" {
            is.update(key, code, event.is_composing(), true);
            event.prevent_default();
        }
    }));
    window().add_event_listener(enclose!( (state) move |event: KeyUpEvent| {
        let is = &mut state.borrow_mut().input;
        let key = event.key();
        let code = event.code();

        if code != "F11" {
            is.update(key, code, event.is_composing(), false);
            event.prevent_default();
        }
    }));

    window().add_event_listener(
        enclose!( (state) move |event: MouseWheelEvent| {
        //        js! {
        //            alert("keydown!");
        //        }
                event.prevent_default();
            }),
    );

    log!("YA");

    render_score(&state.borrow());

    State::run(0.0, state.clone());
    stdweb::event_loop();
}
