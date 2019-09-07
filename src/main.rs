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
            if self.input.keys[Key::Left as usize].press() {
                println!("PRINTLN LEFT");
                self.program.left();
                render_score(self);
            }
            if self.input.keys[Key::Right as usize].press() {
                self.program.right();
                render_score(self);
            }
            if self.input.keys[Key::J as usize].press() {
                self.program.down_step();
                render_score(self);
            }
            if self.input.keys[Key::K as usize].press() {
                self.program.up_step();
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

    let staff_d = score2svg::staff_path_5(SCALEDOWN as i32).d;
    log!("Adding staff: {}", staff_d);
    let staff = js! {
        var staff = document.createElementNS(@{SVGNS}, "path");
        staff.setAttributeNS(null, "d", @{staff_d});
        return staff;
    };

    for path in score2svg::bravura() {
        let id = path.id.unwrap();
        log!("Adding def: {}", id);
        js! {
            var shape = document.createElementNS(@{SVGNS}, "path");
            shape.setAttributeNS(null, "d", @{path.d});
            shape.setAttributeNS(null, "id", @{id});
            @{&defs}.appendChild(shape);
        };
    }
    js! {
        @{&svg}.appendChild(@{&defs});
    }

    let screen_width = SCALEDOWN as i32;
    let mut cursor = score2svg::Cursor::new(state.program.chan,
        state.program.bar, state.program.curs);

    let mut offset_x = 0;
    for measure in 0..9 {
        let mut bar = score2svg::MeasureElem::new(offset_x, 0);
        bar.add_markings(&state.program.scof, state.program.chan, measure,
            &cursor);

        let g = bar.group;
        let measure = js! {
            var g = document.createElementNS(@{SVGNS}, "g");
            g.setAttributeNS(null, "x", @{g.x});
            g.setAttributeNS(null, "y", @{g.y});
            @{&page}.appendChild(g);
            return g;
        };

        for elem in g.elements {
            match elem {
                score2svg::Element::Rect(r) => {
                    log!("Adding rect");
                    js! {
                        var rect = document.createElementNS(@{SVGNS}, "rect");
                        rect.setAttributeNS(null, "x", @{r.x});
                        rect.setAttributeNS(null, "y", @{r.y});
                        rect.setAttributeNS(null, "width", @{r.width});
                        rect.setAttributeNS(null, "height", @{r.height});
                        rect.setAttributeNS(null, "fill", @{r.fill});
                        @{&page}.appendChild(rect);
                    }
                },
                score2svg::Element::Use(u) => {
                    log!("Adding use");
                    let xlink = format!("#{:x}", u.glyph as u32);
                    js! {
                        var stamp = document.createElementNS(@{SVGNS}, "use");
                        stamp.setAttributeNS(null, "x", @{u.x});
                        stamp.setAttributeNS(null, "y", @{u.y});
                        stamp.setAttributeNS(null, "href", @{xlink});
                        @{&page}.appendChild(stamp);
                    }
                },
                score2svg::Element::Path(p) => {
                    log!("Adding path");
                    js! {
                        var shape = document.createElementNS(@{SVGNS}, "path");
                        shape.setAttributeNS(null, "d", @{p.d});
                        @{&page}.appendChild(shape);
                    };
                },
                _ => (),
            }
        }

        offset_x += bar.width;
        if offset_x > screen_width {
            break;
        }
    }

    js! {
        @{&page}.appendChild(@{&staff});
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
