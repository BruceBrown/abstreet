use crate::app::App;
use crate::colors;
use crate::game::{State, Transition, WizardState};
use ezgui::{
    hotkey, Btn, Choice, Composite, EventCtx, GfxCtx, Key, Line, Outcome, TextExt, Widget,
};

// TODO SimOptions stuff too
#[derive(Clone)]
pub struct Options {
    pub traffic_signal_style: TrafficSignalStyle,
    pub color_scheme: Option<String>,
    pub dev: bool,
}

impl Options {
    pub fn default() -> Options {
        Options {
            traffic_signal_style: TrafficSignalStyle::GroupArrows,
            color_scheme: None,
            dev: false,
        }
    }

    fn cs_name(&self) -> &'static str {
        match self.color_scheme {
            Some(ref x) => match x.as_ref() {
                "../data/system/override_colors.json" => "overridden",
                "../data/system/night_colors.json" => "night mode",
                _ => unreachable!(),
            },
            None => "default",
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum TrafficSignalStyle {
    GroupArrows,
    Sidewalks,
    Icons,
    IndividualTurnArrows,
}
impl abstutil::Cloneable for TrafficSignalStyle {}

impl TrafficSignalStyle {
    fn describe(&self) -> &'static str {
        match self {
            TrafficSignalStyle::GroupArrows => {
                "arrows showing the protected and permitted movements"
            }
            TrafficSignalStyle::Sidewalks => {
                "arrows showing the protected and permitted movements, with sidewalks"
            }
            TrafficSignalStyle::Icons => "icons for movements (like the editor UI)",
            TrafficSignalStyle::IndividualTurnArrows => {
                "arrows showing individual turns (to debug)"
            }
        }
    }
}

pub struct OptionsPanel {
    composite: Composite<String>,
    traffic_signal_style: TrafficSignalStyle,
    color_scheme: Option<String>,
}

impl OptionsPanel {
    pub fn new(ctx: &mut EventCtx, app: &App) -> OptionsPanel {
        OptionsPanel {
            composite: Composite::new(
                Widget::col(vec![
                    Widget::row(vec![
                        Line("Settings").roboto_bold().draw(ctx),
                        Btn::text_fg("X")
                            .build_def(ctx, hotkey(Key::Escape))
                            .align_right(),
                    ]),
                    Widget::checkbox(ctx, "Enable developer mode", None, app.opts.dev).margin(5),
                    Widget::checkbox(
                        ctx,
                        "Invert direction of vertical scrolling",
                        None,
                        ctx.canvas.invert_scroll,
                    )
                    .margin(5),
                    Widget::checkbox(
                        ctx,
                        "Use touchpad to pan and hold Control to zoom",
                        None,
                        ctx.canvas.touchpad_to_move,
                    )
                    .margin(5),
                    // TODO Refactor this pattern somehow, using drop-down menus or radio buttons
                    Widget::row(vec![
                        "Traffic signal rendering:".draw_text(ctx).margin(5),
                        Btn::text_fg(format!("{} ▼", app.opts.traffic_signal_style.describe()))
                            .build(ctx, "change traffic signal style", None)
                            .margin(5),
                    ]),
                    Widget::row(vec![
                        "Color scheme:".draw_text(ctx).margin(5),
                        Btn::text_fg(format!("{} ▼", app.opts.cs_name()))
                            .build(ctx, "change color scheme", None)
                            .margin(5),
                    ]),
                    Widget::row(vec![
                        format!(
                            "Scale factor for text / UI elements (your monitor is {}):",
                            ctx.monitor_scale_factor()
                        )
                        .draw_text(ctx)
                        .margin(5),
                        Btn::text_fg(format!("{} ▼", ctx.get_scale_factor()))
                            .build(ctx, "change scale factor", None)
                            .margin(5),
                    ]),
                    Btn::text_bg2("Apply")
                        .build_def(ctx, hotkey(Key::Enter))
                        .margin(5)
                        .centered_horiz(),
                ])
                .bg(colors::PANEL_BG),
            )
            .build(ctx),
            traffic_signal_style: app.opts.traffic_signal_style.clone(),
            color_scheme: app.opts.color_scheme.clone(),
        }
    }
}

impl State for OptionsPanel {
    fn event(&mut self, ctx: &mut EventCtx, app: &mut App) -> Transition {
        match self.composite.event(ctx) {
            Some(Outcome::Clicked(x)) => match x.as_ref() {
                "X" => {
                    return Transition::Pop;
                }
                "Apply" => {
                    ctx.canvas.invert_scroll = self
                        .composite
                        .is_checked("Invert direction of vertical scrolling");
                    ctx.canvas.touchpad_to_move = self
                        .composite
                        .is_checked("Use touchpad to pan and hold Control to zoom");
                    app.opts.dev = self.composite.is_checked("Enable developer mode");

                    if app.opts.traffic_signal_style != self.traffic_signal_style {
                        app.opts.traffic_signal_style = self.traffic_signal_style.clone();
                        println!("Rerendering traffic signals...");
                        for i in app.primary.draw_map.intersections.iter_mut() {
                            *i.draw_traffic_signal.borrow_mut() = None;
                        }
                    }

                    if app.opts.color_scheme != self.color_scheme {
                        app.opts.color_scheme = self.color_scheme.take();
                        app.switch_map(ctx, app.primary.current_flags.sim_flags.load.clone());
                    }

                    return Transition::Pop;
                }
                "change traffic signal style" => {
                    return Transition::Push(WizardState::new(Box::new(|wiz, ctx, _| {
                        let (_, style) =
                            wiz.wrap(ctx)
                                .choose("How should traffic signals be drawn?", || {
                                    vec![
                                        Choice::new(
                                            TrafficSignalStyle::GroupArrows.describe(),
                                            TrafficSignalStyle::GroupArrows,
                                        ),
                                        Choice::new(
                                            TrafficSignalStyle::Sidewalks.describe(),
                                            TrafficSignalStyle::Sidewalks,
                                        ),
                                        Choice::new(
                                            TrafficSignalStyle::Icons.describe(),
                                            TrafficSignalStyle::Icons,
                                        ),
                                        Choice::new(
                                            TrafficSignalStyle::IndividualTurnArrows.describe(),
                                            TrafficSignalStyle::IndividualTurnArrows,
                                        ),
                                    ]
                                })?;
                        Some(Transition::PopWithData(Box::new(move |state, _, ctx| {
                            let mut panel = state.downcast_mut::<OptionsPanel>().unwrap();
                            panel.composite.replace(
                                ctx,
                                "change traffic signal style",
                                Btn::text_fg(format!("{} ▼", style.describe()))
                                    .build(ctx, "change traffic signal style", None)
                                    .margin(5),
                            );
                            panel.traffic_signal_style = style;
                        })))
                    })));
                }
                "change color scheme" => {
                    return Transition::Push(WizardState::new(Box::new(|wiz, ctx, _| {
                        let (descr, path) = wiz.wrap(ctx).choose("What color scheme?", || {
                            vec![
                                Choice::new("default", None),
                                Choice::new(
                                    "overridden colors",
                                    Some("../data/system/override_colors.json".to_string()),
                                ),
                                Choice::new(
                                    "night mode",
                                    Some("../data/system/night_colors.json".to_string()),
                                ),
                            ]
                        })?;
                        Some(Transition::PopWithData(Box::new(move |state, _, ctx| {
                            let mut panel = state.downcast_mut::<OptionsPanel>().unwrap();
                            panel.composite.replace(
                                ctx,
                                "change color scheme",
                                Btn::text_fg(format!("{} ▼", descr))
                                    .build(ctx, "change color scheme", None)
                                    .margin(5),
                            );
                            panel.color_scheme = path;
                        })))
                    })));
                }
                "change scale factor" => {
                    return Transition::Push(WizardState::new(Box::new(|wiz, ctx, _| {
                        let (_, scale) = wiz.wrap(ctx).choose(
                            "What scale factor for text / UI elements?",
                            || {
                                vec![
                                    Choice::new("0.5", 0.5),
                                    Choice::new("1.0", 1.0),
                                    Choice::new("1.5", 1.5),
                                    Choice::new("2.0", 2.0),
                                ]
                            },
                        )?;
                        Some(Transition::PopWithData(Box::new(move |state, _, ctx| {
                            let panel = state.downcast_mut::<OptionsPanel>().unwrap();
                            panel.composite.replace(
                                ctx,
                                "change scale factor",
                                Btn::text_fg(format!("{} ▼", scale))
                                    .build(ctx, "change scale factor", None)
                                    .margin(5),
                            );
                            ctx.set_scale_factor(scale);
                        })))
                    })));
                }
                _ => unreachable!(),
            },
            None => {}
        }

        Transition::Keep
    }

    fn draw(&self, g: &mut GfxCtx, _: &App) {
        self.composite.draw(g);
    }
}
