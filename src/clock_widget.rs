use std::f64::consts::PI;
use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Timelike, Utc};
use druid::kurbo::{Circle, Line};
use druid::piet::{FontBuilder, Text, TextLayout, TextLayoutBuilder};
use druid::theme;
use druid::widget::Widget;
use druid::{
    BoxConstraints, Color, EventCtx, LayoutCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx,
    Vec2,
};
use druid::{Data, Env, Event};

/// A Druid Clock Widget
///
///
/// ```compile_fail rust
///     //This code does not compile because the Closure parameter (&UiModel) is not in context...
///     use druid::widget::Flex;
///     use clock_widget::{Clock, ClockConfigData, ClockColors};
///     use chrono::FixedOffset;
///     let mut r = Flex::row();
///         r.add_child(
///            Clock::new(
///                move |model: &UiModel| ClockConfigData::new(
///                    Some(String::from("UTC")),
///                    FixedOffset::east(0),
///                    None,
///                    Some(ClockColors::default()),
///                ),
///                400.0,
///                true,
///            ),
///            1.0,
///        );
/// ```
///
pub struct Clock<T> {
    size: Size,
    config: ClockConfigType<T>,
    send_timer_requests: bool,
}

/// Data to configure the Clock;
/// - An optional Name that sit on top of the clock.
/// - The clock's time-offset
/// - An optional date-format: See https://docs.rs/chrono/0.4.10/chrono/format/strftime/index.html#specifiers
///   It defaults to  "%F %T %:z" e.g. "2020-01-27 20:30:15 +13:00"
pub struct ClockConfigData {
    name: Option<String>,
    utc_offset: FixedOffset,
    date_format: Option<String>,
    colors: Option<ClockColors>,
}

#[derive(Clone, Debug)]
pub struct ClockColors {
    col_hour_hand: Color,
    col_minute_hand: Color,
    col_second_hand: Color,
}

impl<T: Data> Clock<T> {
    pub fn new(
        config: impl Into<ClockConfigType<T>>,
        diameter: f64,
        send_timer_requests: bool,
    ) -> Self {
        Clock {
            size: Size::from((diameter, diameter + 60.0)),
            config: config.into(),
            send_timer_requests,
        }
    }
}

pub enum ClockConfigType<T> {
    Specific(ClockConfigData),
    Dynamic(Box<dyn Fn(&T) -> ClockConfigData>),
}

impl Default for ClockColors {
    fn default() -> Self {
        Self {
            col_hour_hand: Color::rgb8(255, 0, 125),
            col_minute_hand: Color::rgb8(125, 0, 255),
            col_second_hand: Color::rgb8(125, 0, 125),
        }
    }
}

impl<T: Data> ClockConfigType<T> {
    pub fn get_data<V>(&self, data: &T, mut cb: impl FnMut(&ClockConfigData) -> V) -> V {
        match self {
            ClockConfigType::Specific(c) => cb(c),
            ClockConfigType::Dynamic(f) => cb(&(f)(data)),
        }
    }
}

impl ClockConfigData {
    pub fn new(
        name: Option<String>,
        utc_offset: FixedOffset,
        date_format: Option<String>,
        colors: Option<ClockColors>,
    ) -> ClockConfigData {
        ClockConfigData {
            name,
            utc_offset,
            date_format,
            colors,
        }
    }
}

impl<T: Data> Widget<T> for Clock<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut T, _env: &Env) {
        match event {
            Event::LifeCycle(lc) => {
                println!("Starting Clock: {:?}", lc);

                //The next request has to be aligned with the system clock, so that the second-hand moves in step...
                ctx.request_timer(
                    Instant::now()
                        + Duration::from_millis(1000 - Utc::now().timestamp_subsec_millis() as u64),
                );
            }
            Event::Timer(_timer) => {
                ctx.set_handled();
                if self.send_timer_requests {
                    ctx.request_anim_frame();
                    ctx.request_timer(
                        Instant::now()
                            + Duration::from_millis(
                                1000 - Utc::now().timestamp_subsec_millis() as u64,
                            ),
                    );
                }
            }
            Event::Size(size) => {
                self.size = *size;
            }
            _ => (),
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: Option<&T>, _data: &T, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        _bc: &BoxConstraints,
        _data: &T,
        _env: &Env,
    ) -> Size {
        self.size
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &T, env: &Env) {
        //The Date-Time...
        let mut now = DateTime::<FixedOffset>::from(Utc::now());
        &self.config.get_data(data, |d| {
            now = now.with_timezone::<FixedOffset>(&d.utc_offset)
        });

        let font_name = env.get(theme::FONT_NAME);
        let font_size = env.get(theme::TEXT_SIZE_NORMAL);

        let mut name = String::from("");
        let mut date_format = String::from("%F %T %:z");
        let mut colors = ClockColors::default();

        //An Optional Clock-name...
        self.config.get_data(data, |d| {
            if let Some(n) = &d.name {
                name = n.clone();
            }
            if let Some(df) = &d.date_format {
                date_format = df.clone();
            }
            if let Some(c) = &d.colors {
                colors = c.clone();
            }
        });
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

        let minute = centre
            + rad * 245.0 / 265.0 * Vec2::from_angle(get_angle(now.minute().into(), 60, true));
        let sec = centre
            + rad * 260.0 / 265.0 * Vec2::from_angle(get_angle(now.second().into(), 60, true));
        let time_font_size = font_size * f64::min(self.size.height, self.size.width) / 275.0;

        let sec_tip_radius = f64::max(8.0, rad * 12.0 / 265.0);

        let second_color = with_gamma(
            &colors.col_second_hand,
            0.2 + 0.2 * now.second() as f64 / 60.0,
        );

        //The Face...
        paint_ctx.fill(
            Circle::new(centre, rad),
            &Color::rgb8(180, 200, 180).with_alpha(0.7),
        );
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
        paint_ctx.fill(
            Circle::new(hour, sec_tip_radius * 1.3),
            &colors.col_hour_hand,
        );
        paint_ctx.fill(Circle::new(sec, sec_tip_radius), &colors.col_second_hand);

        paint_ctx.stroke(
            Line::new(centre, hour),
            &colors.col_hour_hand,
            f64::max(5.0, rad * 20.0 / 265.0),
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
            &colors.col_minute_hand,
            f64::max(4.0, rad * 10.0 / 265.0),
        );
        paint_ctx.stroke(
            Line::new(centre, sec),
            &colors.col_second_hand,
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
        let tl3 = t
            .new_text_layout(&f, &name)
            .build()
            .expect("Ai can notti di maik");

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

        paint_ctx.draw_text(
            &tl3,
            Point::new(f64::max(0.0, centre_x - tl3.width() / 2.0), 5.0),
            &Color::rgb8(255, 255, 125),
        );
    }
}

///Determines the radians for a given arc-count and how many arcs fit in one revolution.
/// e.g. 30 seconds would yield 30/60*2Pi;
/// In order to have zero-arc-count point upward and not forward, a rotation of -1/2PI is to added as an offset
fn get_angle(arc_count: f64, arcs_per_revolution: u64, add_offset: bool) -> f64 {
    (if add_offset { -0.25 } else { 0.0 } + arc_count / arcs_per_revolution as f64) * 2.0 * PI
}

impl<T, F: Fn(&T) -> ClockConfigData + 'static> From<F> for ClockConfigType<T> {
    fn from(src: F) -> ClockConfigType<T> {
        ClockConfigType::Dynamic(Box::new(src))
    }
}

impl<T> From<ClockConfigData> for ClockConfigType<T> {
    fn from(src: ClockConfigData) -> ClockConfigType<T> {
        ClockConfigType::Specific(src)
    }
}

impl From<[Color; 3]> for ClockColors {
    fn from(src: [Color; 3]) -> ClockColors {
        ClockColors {
            col_hour_hand: src[0].clone(),
            col_minute_hand: src[1].clone(),
            col_second_hand: src[2].clone(),
        }
    }
}

fn with_gamma(c: &Color, gamma: f64) -> Color {
    let rgba = c.as_rgba_u32();
    Color::rgb8(
        apply_gamma(rgba >> 24, gamma),
        apply_gamma(rgba >> 16, gamma),
        apply_gamma(rgba >> 8, gamma),
    )
}

fn apply_gamma(base_color: u32, gamma: f64) -> u8 {
    ((base_color % 256) as f64 * f64::min(1.0, gamma)) as u8
}
