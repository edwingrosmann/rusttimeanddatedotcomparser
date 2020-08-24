use std::f64::consts::PI;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Timelike};
use druid::kurbo::{Circle, Line};
use druid::piet::{FontBuilder, StrokeStyle, Text, TextLayout, TextLayoutBuilder};
use druid::widget::Widget;
use druid::{theme, AppDelegate, DelegateCtx, WindowId};
use druid::{AppLauncher, Command, Data, Env, Event, WindowDesc};
use druid::{
    BoxConstraints, Color, EventCtx, LayoutCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx,
    Vec2,
};

///                              _______________________
///              ,~~~~,         /                       \    
///             //(??)\\ -=====( Oh Druid Please Show Us )
///             ~ _)(_ ~        \_______________________/
///            //(0 0)\\
///            \| ).( |/
///             @//^\\@
///              \| |/
///            `~~  ~~'
pub fn show(test_mode: bool) {
    AppLauncher::with_window(WindowDesc::new(|| UiModel::default()).window_size((560.0, 600.0)))
        .delegate(UiModel {
            test_mode,
            ..Default::default()
        })
        .launch(UiModel::default())
        .expect("launch failed");
}

///This is both the Data AND the Delegate
#[derive(Debug, Clone)]
struct UiModel {
    size: Size,
    col_hour_hand: Color,
    col_minute_hand: Color,
    col_second_hand: Color,
    test_mode: bool,
}

impl Default for UiModel {
    fn default() -> Self {
        UiModel {
            size: Size::default(),
            col_hour_hand: Color::rgb8(255, 0, 125),
            col_minute_hand: Color::rgb8(125, 0, 255),
            col_second_hand: Color::rgb8(125, 0, 125),
            test_mode: false,
        }
    }
}

impl Data for UiModel {
    fn same(&self, _other: &Self) -> bool {
        true
    }
}

impl Widget<UiModel> for UiModel {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut UiModel, _env: &Env) {
        match event {
            Event::LifeCycle(lc) => {
                println!("Starting Clock App: {:?}", lc);
                //The first request has to be aligned with the system clock, so that the second-hand moves in step...
                ctx.request_timer(
                    Instant::now()
                        + Duration::from_millis(
                            1000 - Local::now().timestamp_subsec_millis() as u64,
                        ),
                );
            }
            Event::Timer(_timer) => {
                //                ctx.set_handled();
                ctx.request_timer(
                    Instant::now()
                        + Duration::from_millis(
                            1000 - Local::now().timestamp_subsec_millis() as u64,
                        ),
                );
                ctx.request_anim_frame();
            }
            Event::Size(size) => {
                self.size = *size;
            }
            _ => (),
        }
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: Option<&UiModel>,
        _data: &UiModel,
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &UiModel,
        _env: &Env,
    ) -> Size {
        bc.constrain((550.0, 860.0))
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &UiModel, env: &Env) {
        //The Date-Time...
        let now = DateTime::<Local>::from(Local::now());

        let font_name = env.get(theme::FONT_NAME);
        let font_size = env.get(theme::TEXT_SIZE_NORMAL);
        //
        //determine the dimensions...
        let centre_x = self.size.width / 2.0;
        let centre_y = (self.size.height - 50.0) / 2.0;
        let rad = f64::min(centre_x, centre_y) - 15.0;
        let centre = Point::new(centre_x, centre_y);
        //
        //The hour-angle is to be adjusted with 1/12 of the minute-angle...
        let hour = centre
            + rad * 145.0 / 265.0
                * Vec2::from_angle(
                    get_angle(now.hour().into(), 12, true)
                        + get_angle(now.minute().into(), 12 * 60, false),
                );
        let hour_ext = centre
            + rad
                * Vec2::from_angle(
                    get_angle(now.hour().into(), 12, true)
                        + get_angle(now.minute().into(), 12 * 60, false),
                );
        let minute = centre
            + rad * 245.0 / 265.0 * Vec2::from_angle(get_angle(now.minute().into(), 60, true));
        let sec = centre
            + rad * 260.0 / 265.0 * Vec2::from_angle(get_angle(now.second().into(), 60, true));
        let time_font_size = font_size * f64::min(self.size.height, self.size.width) / 275.0;

        let sec_tip_radius = f64::max(8.0, rad * 12.0 / 265.0);
        let second_color = Color::rgb8(
            (40 + now.second() / 2) as u8,
            0,
            (40 + now.second() / 2) as u8,
        );

        //The Face...
        paint_ctx.fill(Circle::new(centre, rad), &Color::rgb8(180, 200, 180));
        for i in 0..60 {
            let mark = centre + rad * Vec2::from_angle(get_angle(i.into(), 60, false));
            paint_ctx.stroke(Line::new(centre, mark), &Color::rgb8(50, 0, 50), 3.0);
        }
        for i in 0..12 {
            let mark = centre + rad * Vec2::from_angle(get_angle(i.into(), 12, false));
            paint_ctx.stroke(Line::new(centre, mark), &Color::rgb8(50, 0, 50), 6.0);
        }
        paint_ctx.fill(
            Circle::new(centre, f64::max(3.0, rad * 255.0 / 265.0)),
            &second_color,
        );
        //
        //Three-arms...
        paint_ctx.fill(Circle::new(hour, sec_tip_radius * 1.3), &self.col_hour_hand);
        paint_ctx.fill(Circle::new(sec, sec_tip_radius), &self.col_second_hand);

        paint_ctx.stroke(
            Line::new(centre, hour),
            &self.col_hour_hand,
            f64::max(5.0, rad * 20.0 / 265.0),
        );

        let style = &mut StrokeStyle::new();
        style.set_dash([5.0, 10.0].to_vec(), 2.0);
        paint_ctx.stroke_styled(
            Line::new(hour, hour_ext),
            &data.col_hour_hand,
            f64::max(1.0, rad * 2.0 / 265.0),
            style,
        );

        //The Text in the Hour-hand-tip should be UNDER the minute and hour hand...
        let t = paint_ctx.text();

        let f = t
            .new_font_by_name(font_name, f64::max(time_font_size * 0.45, font_size * 0.55))
            .build()
            .expect("No Font Found");

        let tl = t
            .new_text_layout(&f, &now.format("%p").to_string().as_str())
            .build()
            .expect("Ai impooohssibile");

        paint_ctx.draw_text(
            &tl,
            Point::new(
                hour.x - sec_tip_radius * 0.92,
                hour.y + sec_tip_radius * 0.45,
            ),
            &Color::rgb8(255, 255, 0),
        );
        paint_ctx.stroke(
            Line::new(centre, minute),
            &self.col_minute_hand,
            f64::max(4.0, rad * 10.0 / 265.0),
        );
        paint_ctx.stroke(
            Line::new(centre, sec),
            &self.col_second_hand,
            f64::max(3.0, rad * 6.0 / 265.0),
        );
        //The centre-pin...
        paint_ctx.fill(
            Circle::new(centre, f64::max(6.0, rad * 14.0 / 265.0)),
            &Color::rgb8(200, 100, 200),
        );
        paint_ctx.fill(
            Circle::new(centre, f64::max(1.5, rad * 5.0 / 265.0)),
            &second_color,
        );

        let date_format = String::from("%F %T %:z");

        //All the Text-items that should sit on TOP...
        let t = paint_ctx.text();

        let f = t
            .new_font_by_name(font_name, time_font_size.min(50.0))
            .build()
            .expect("No Font Found");
        let f2 = t
            .new_font_by_name(font_name, f64::max(time_font_size * 0.4, font_size * 0.55))
            .build()
            .expect("No Font Found");

        let tl = t
            .new_text_layout(&f, &now.format(&date_format).to_string().as_str())
            .build()
            .expect("Ai can notti di maik");
        let tl2 = t
            .new_text_layout(&f2, format!("{:02}", now.second()).to_string().as_str())
            .build()
            .expect("Ai impooohssibile");

        paint_ctx.draw_text(
            &tl,
            Point::new(
                f64::max(0.0, centre_x - tl.width() / 2.0),
                //This calculation determines the halfway point of
                //the remaining space below the clock plus the font-height; it looks pretty sad I know :-)
                // Something like:
                //   centre-x+rad [is bottom of clock] +
                //   (self.size.height-centre-x-rad)*2/3 [is a two-third of the height below the clock] +
                //   2*time_font_size/font_size [an extra offset based on the font-size]
                // The terms can then be simplified to...
                self.size.height / 1.5
                    + centre.y / 3.0
                    + rad / 3.0
                    + 2.0 * time_font_size / font_size,
            ),
            &Color::rgb8(255, 125, 125),
        );
        paint_ctx.draw_text(
            &tl2,
            Point::new(
                sec.x - sec_tip_radius * 0.655,
                sec.y + sec_tip_radius * 0.435,
            ),
            &Color::rgb8(255, 255, 0),
        );
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

///Determines the radians for a given arc-count and how many arcs fit in one revolution.
/// e.g. 30 seconds would yield 30/60*2Pi;
/// In order to have zero-arc-count point upward and not forward, a rotation of -1/2PI is to added as an offset
fn get_angle(arc_count: f64, arcs_per_revolution: u64, add_offset: bool) -> f64 {
    (if add_offset { -0.25 } else { 0.0 } + arc_count / arcs_per_revolution as f64) * 2.0 * PI
}
