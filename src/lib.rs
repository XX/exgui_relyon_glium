extern crate nanovg;
extern crate exgui;

use std::collections::HashMap;
use std::path::Path;
use nanovg::{Context, ContextBuilder, Font, Frame, Color as NanovgColor,
             StrokeOptions, LineCap as NanovgLineCap, LineJoin as NanovgLineJoin, PathOptions};
use exgui::{Node, ModelComponent, Drawable, Shape, Color, Stroke, LineCap, LineJoin};

pub trait AsNanovgColor {
    fn as_nanovg_color(&self) -> NanovgColor;
}

impl AsNanovgColor for Color {
    fn as_nanovg_color(&self) -> NanovgColor {
        let [r, g, b, a] = self.as_arr();
        NanovgColor::new(r, g, b, a)
    }
}

pub struct Renderer<'a> {
    pub context: Context,
    pub fonts: HashMap<String, Font<'a>>,
    pub width: f32,
    pub height: f32,
    pub device_pixel_ratio: f32,
}

impl<'a> Renderer<'a> {
    pub fn new() -> Self {
        let context = ContextBuilder::new()
            .stencil_strokes()
            .build()
            .expect("Initialization of NanoVG context failed!");

        let renderer = Self::new_with_context(context);
        //renderer.load_font("Roboto", "resources/Roboto-Regular.ttf");
        renderer
    }

    pub fn new_with_context(context: Context) -> Self {
        Renderer {
            context,
            fonts: HashMap::new(),
            width: 0.0,
            height: 0.0,
            device_pixel_ratio: 0.0,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn with_device_pixel_ratio(mut self, device_pixel_ratio: f32) -> Self {
        self.device_pixel_ratio = device_pixel_ratio;
        self
    }

    pub fn load_font<S: Into<String>, P: AsRef<Path>>(&'a mut self, name: S, path: P) {
        let name = name.into();
        let display_path = path.as_ref().display();
        let font = Font::from_file(&self.context, name.as_str(), path.as_ref())
            .expect(&format!("Failed to load font '{}'", display_path));
        self.fonts.insert(name, font);
    }

    pub fn render<MC: ModelComponent>(&self, node: &Node<MC>) {
        self.context.frame(
            (self.width, self.height),
            self.device_pixel_ratio,
            Self::render_handler(node)
        );
    }

    pub fn render_handler<MC: ModelComponent>(node: &'a Node<MC>) -> impl FnOnce(Frame<'a>) {
        move |frame| {
            Self::render_draw(&frame, node as &dyn Drawable);
        }
    }

    fn render_draw(frame: &Frame<'a>, draw: &dyn Drawable) {
        if let Some(shape) = draw.shape() {
            match shape {
                Shape::Rect(ref r) => {
                    frame.path(
                        |path| {
                            path.rect((r.x, r.y), (r.width, r.height));
                            if let Some(fill) = r.fill {
                                path.fill(fill.color.as_nanovg_color(), Default::default());
                            };
                            if let Some(stroke) = r.stroke {
                                path.stroke(
                                    stroke.color.as_nanovg_color(),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        PathOptions::default(),
                    );
                },
                Shape::Circle(ref c) => {
                    frame.path(
                        |path| {
                            path.circle((c.cx, c.cy), c.r);
                            if let Some(fill) = c.fill {
                                path.fill(fill.color.as_nanovg_color(), Default::default());
                            };
                            if let Some(stroke) = c.stroke {
                                path.stroke(
                                    stroke.color.as_nanovg_color(),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        PathOptions::default(),
                    );
                },
                Shape::Path(ref p) => {
                    frame.path(
                        |path| {
                            use exgui::PathCommand::*;

                            let mut last_xy = [0.0_f32, 0.0];
                            let mut bez_ctrls = [(0.0_f32, 0.0), (0.0_f32, 0.0)];

                            for cmd in p.cmd.iter() {
                                match cmd {
                                    Move(ref xy) => {
                                        last_xy = *xy;
                                        path.move_to((last_xy[0], last_xy[1]));
                                    },
                                    MoveRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.move_to((last_xy[0], last_xy[1]));
                                    },
                                    Line(ref xy) => {
                                        last_xy = *xy;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonX(ref x) => {
                                        last_xy[0] = *x;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonXRel(ref x) => {
                                        last_xy[0] += *x;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonY(ref y) => {
                                        last_xy[1] = *y;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonYRel(ref y) => {
                                        last_xy[1] += *y;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    Close => path.close(),
                                    BezCtrl(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (xy[0], xy[1])];
                                    },
                                    BezCtrlRel(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (last_xy[0] + xy[0], last_xy[1] + xy[1])];
                                    },
                                    QuadBezTo(ref xy) => {
                                        last_xy = *xy;
                                        path.quad_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[1]);
                                    },
                                    QuadBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.quad_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[1]);
                                    },
                                    CubBezTo(ref xy) => {
                                        last_xy = *xy;
                                        path.cubic_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[0], bez_ctrls[1]);
                                    },
                                    CubBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.cubic_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[0], bez_ctrls[1]);
                                    },
                                    _ => panic!("Not impl rendering cmd {:?}", cmd), // TODO: need refl impl
                                }
                            }
                            if let Some(fill) = p.fill {
                                path.fill(fill.color.as_nanovg_color(), Default::default());
                            };
                            if let Some(stroke) = p.stroke {
                                path.stroke(
                                    stroke.color.as_nanovg_color(),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        PathOptions::default(),
                    );
                },
                Shape::Group(ref _g) => {},
            }
        }
        if let Some(childs) = draw.childs() {
            for child in childs {
                Self::render_draw(frame, child);
            }
        }
    }

    fn stroke_option(stroke: &Stroke) -> StrokeOptions {
        let line_cap = match stroke.line_cap {
            LineCap::Butt => NanovgLineCap::Butt,
            LineCap::Round => NanovgLineCap::Round,
            LineCap::Square => NanovgLineCap::Square,
        };
        let line_join = match stroke.line_join {
            LineJoin::Miter => NanovgLineJoin::Miter,
            LineJoin::Round => NanovgLineJoin::Round,
            LineJoin::Bevel => NanovgLineJoin::Bevel,
        };
        StrokeOptions {
            width: stroke.width,
            line_cap,
            line_join,
            miter_limit: stroke.miter_limit,
            ..Default::default()
        }
    }
}
