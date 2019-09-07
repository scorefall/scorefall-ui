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

use scof::Cursor;
use scorefall::Program;

mod input;

use input::*;

const SCALEDOWN: f64 = 25_000.0;
const SVGNS: &str = "http://www.w3.org/2000/svg";

struct State {
    program: Program,
    time_old: f64,
    command: String,
    input: InputState,
    svg: stdweb::web::Element,
}

impl State {
    /// Create a new state
    fn new(svg: stdweb::web::Element) -> State {
        State {
            program: Program::new(),
            time_old: 0.0,
            command: "".to_string(),
            input: InputState::new(),
            svg,
        }
    }

    /// Resize the SVG
    fn resize(&self) {
        log!("resize");
        let svg = &self.svg;
        js! {
            var svg = @{svg};
            var ratio = svg.clientHeight / svg.clientWidth;
            var viewbox = "0 0 " + @{SCALEDOWN} + " " + (@{SCALEDOWN} * ratio);
            svg.setAttributeNS(null, "viewBox", viewbox);
        }
    }

    fn process_input(&mut self, time: f64) {
        let _dt = (time - self.time_old) as f32;
        self.time_old = time;

        if self.input.has_input {
            if self.input.keys[Key::Left as usize].press() {
                self.program.left();
                self.render_measures();
            }
            if self.input.keys[Key::Right as usize].press() {
                self.program.right();
                self.render_measures();
            }
            if self.input.keys[Key::LeftShift as usize].held() || self.input.keys[Key::RightShift as usize].held()
            {
                if self.input.keys[Key::J as usize].press() {
                    self.program.down_half_step();
                    self.render_measures();
                }
                if self.input.keys[Key::K as usize].press() {
                    self.program.up_half_step();
                    self.render_measures();
                }
            } else {
                if self.input.keys[Key::J as usize].press() {
                    self.program.down_step();
                    self.render_measures();
                }
                if self.input.keys[Key::K as usize].press() {
                    self.program.up_step();
                    self.render_measures();
                }
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

    /// Initialize the score SVG
    fn initialize_score(&self) {
        let svg = &self.svg;

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

        let ratio = h / w;
        let viewbox = format!("0 0 {} {}", SCALEDOWN, SCALEDOWN * ratio);
        js! {
            var svg = document.getElementById("canvas");
            svg.setAttributeNS(null, "viewBox", @{viewbox});
        };
    }

    /// Render the defs to the SVG
    fn render_defs(&self) {
        let svg = js! { return document.getElementById("canvas"); };
        let defs = js! { return document.createElementNS(@{SVGNS}, "defs"); };

        for path in score2svg::bravura() {
            let id = path.id.unwrap();
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
    }

    /// Render the score
    fn render_score(&self) {
        self.initialize_score();
        self.render_defs();
        js! {
            var svg = document.getElementById("canvas");
            var page = document.createElementNS(@{SVGNS}, "g");
            page.setAttributeNS(null, "id", "page");
            svg.appendChild(page);
        };
        self.render_measures();
    }

    /// Render the measures to the SVG
    fn render_measures(&self) {
        log!("render measures");
        let page = js! {
            var svg = document.getElementById("canvas");
            var page = svg.getElementById("page");
            page.innerHTML = "";
            return page;
        };

        let mut offset_x = 0;
        for measure in 0..9 {
            let width = self.render_measure(measure, offset_x,
                &self.program.cursor);
            log!("measure: {}  width {}", measure, width);
            offset_x += width;
        }
    }

    fn render_measure(&self, measure: usize, offset_x: i32, cursor: &Cursor)
        -> i32
    {
        let bar_id = &format!("m{}", measure);
        let offset_y = 0;
        let trans = &format!("translate({} {})", offset_x, offset_y);
        let bar_g = js! {
            var svg = document.getElementById("canvas");
            var page = svg.getElementById("page");
            var old_g = svg.getElementById(@{bar_id});
            var bar_g = document.createElementNS(@{SVGNS}, "g");
            bar_g.setAttributeNS(null, "id", @{bar_id});
            bar_g.setAttributeNS(null, "transform", @{trans});
            if (old_g !== null) {
                old_g.replaceWith(bar_g);
            } else {
                page.appendChild(bar_g);
            }
            return bar_g;
        };

        let mut curs = Cursor::new(measure, 0, 0);
        let mut bar = score2svg::MeasureElem::new();
        bar.add_markings(&self.program.scof, &mut curs, &cursor);
        bar.add_staff_5();

        for elem in bar.elements {
            match elem {
                score2svg::Element::Rect(r) => {
                    js! {
                        var rect = document.createElementNS(@{SVGNS}, "rect");
                        rect.setAttributeNS(null, "x", @{r.x});
                        rect.setAttributeNS(null, "y", @{r.y});
                        rect.setAttributeNS(null, "width", @{r.width});
                        rect.setAttributeNS(null, "height", @{r.height});
                        rect.setAttributeNS(null, "fill", @{r.fill});
                        @{&bar_g}.appendChild(rect);
                    }
                },
                score2svg::Element::Use(u) => {
                    let xlink = format!("#{:x}", u.glyph as u32);
                    js! {
                        var stamp = document.createElementNS(@{SVGNS}, "use");
                        stamp.setAttributeNS(null, "x", @{u.x});
                        stamp.setAttributeNS(null, "y", @{u.y});
                        stamp.setAttributeNS(null, "href", @{xlink});
                        @{&bar_g}.appendChild(stamp);
                    }
                },
                score2svg::Element::Path(p) => {
                    js! {
                        var shape = document.createElementNS(@{SVGNS}, "path");
                        shape.setAttributeNS(null, "d", @{p.d});
                        @{&bar_g}.appendChild(shape);
                    };
                },
                _ => (),
            }
        }
        bar.width
    }
}

fn main() {
    stdweb::initialize();

    let svg = document().get_element_by_id("canvas").unwrap();
    let state = Rc::new(RefCell::new(State::new(svg)));

    // FIXME: Use this.
    let _prompt: stdweb::web::Element =
        document().get_element_by_id("prompt").unwrap();

    window().add_event_listener(enclose!( (state) move |_: ResizeEvent| {
        state.borrow().resize();
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

    state.borrow().render_score();

    State::run(0.0, state.clone());
    stdweb::event_loop();
}
