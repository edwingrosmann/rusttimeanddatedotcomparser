use std::f64::consts::PI;
use std::time::{Duration, Instant};

use druid::kurbo::Line;
use druid::piet::{FontBuilder, Text, TextLayoutBuilder};
use druid::widget::Widget;
use druid::{theme, AppDelegate, DelegateCtx, WindowId};
use druid::{AppLauncher, Data, Env, Event, WindowDesc};
use druid::{
    BoxConstraints, Color, Command, EventCtx, LayoutCtx, PaintCtx, Point, RenderContext, Size,
    UpdateCtx, Vec2,
};

#[derive(Debug, Clone, Default)]
struct UiModel {
    t: f64,
    test_mode: bool,
}

impl Data for UiModel {
    fn same(&self, _: &Self) -> bool {
        true
    }
}

pub fn show_animation(test_mode: bool) {
    AppLauncher::with_window(
        WindowDesc::new(move || UiModel {
            test_mode,
            ..Default::default()
        })
        .window_size((1000.0, 500.0)),
    )
    .delegate(UiModel {
        test_mode,
        ..Default::default()
    })
    .launch(UiModel::default())
    .expect("launch failed");
}

impl Widget<UiModel> for UiModel {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut UiModel, _env: &Env) {
        match event {
            Event::LifeCycle(lc) => {
                println!("Starting UI: {:?}", lc);
                if self.test_mode {
                    self.t = 2.1;
                    ctx.request_anim_frame();
                }
                ctx.request_timer(Instant::now() + Duration::from_secs(1));
            }
            Event::MouseDown(_) => {
                self.t = 0.0;
                ctx.request_anim_frame();
            }
            Event::AnimFrame(interval) => {
                self.t += (*interval as f64) * 5e-10;
                if self.t < 3.00000 {
                    ctx.request_anim_frame();
                }
            }
            _ => {
                if self.test_mode {
                    self.t = 2.1;
                    ctx.request_anim_frame();
                }
            }
        }
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: Option<&UiModel>,
        _data: &UiModel,
        _env: &Env,
    ) {
        println!("Updating: {}", self.t);
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &UiModel,
        _env: &Env,
    ) -> Size {
        bc.constrain((650.0, 650.0))
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, _data: &UiModel, env: &Env) {
        let center = Point::new(150.0, 150.0);
        let ambit = center + 145.0 * Vec2::from_angle((0.75 + self.t) * 2.0 * PI);
        paint_ctx.stroke(Line::new(center, ambit), &Color::rgb8(255, 0, 125), 20.0);
        let center2 = Point::new(350.0, 150.0);
        let ambit2 = center2 + 145.0 * Vec2::from_angle((-0.75 - self.t) * 2.0 * PI);
        paint_ctx.stroke(Line::new(center2, ambit2), &Color::rgb8(125, 0, 255), 10.0);
        paint_ctx.stroke(Line::new(ambit, ambit2), &Color::rgb8(255, 0, 255), 10.0);
        let center2 = Point::new(150.0, 150.0);
        let ambit2 = center2 + 145.0 * Vec2::from_angle((-0.75 - self.t) * 2.0 * PI);
        paint_ctx.stroke(Line::new(center2, ambit2), &Color::rgb8(0, 125, 255), 10.0);
        let center = Point::new(350.0, 150.0);
        let ambit = center + 145.0 * Vec2::from_angle((0.75 + self.t) * 2.0 * PI);
        paint_ctx.stroke(Line::new(center, ambit), &Color::rgb8(0, 255, 125), 20.0);
        paint_ctx.stroke(Line::new(ambit, ambit2), &Color::rgb8(0, 255, 255), 10.0);

        let font_name = env.get(theme::FONT_NAME);
        let font_size = env.get(theme::TEXT_SIZE_NORMAL) * 2.0;
        let t = paint_ctx.text();
        let f = t
            .new_font_by_name(font_name, font_size)
            .build()
            .expect("Notte di Font no found");
        let tl = t
            .new_text_layout(&f, "Howdi-doo-di")
            .build()
            .expect("Ai can notti di maik");

        let tl2 = t
            .new_text_layout(&f, format!("{}", ambit).as_str())
            .build()
            .expect("Aiij can notti di maik");

        paint_ctx.draw_text(&tl, Point::new(150.0, 350.0), &Color::rgb8(255, 125, 125));
        paint_ctx.draw_text(&tl2, Point::new(150.0, 450.0), &Color::rgb8(0, 225, 125));
    }
}
impl AppDelegate<UiModel> for UiModel {
    fn window_added(
        &mut self,
        id: WindowId,
        _data: &mut UiModel,
        _env: &Env,
        ctx: &mut DelegateCtx,
    ) {
        println!("Window Added: {:?}", id);
        if self.test_mode {
            ctx.submit_command(Command::from(druid::commands::CLOSE_WINDOW), id);
        }
    }
}
