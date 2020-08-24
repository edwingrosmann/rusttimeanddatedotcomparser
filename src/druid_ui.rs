use std::collections::HashMap;
use std::process::Command as Cmd;

use chrono::{DateTime, FixedOffset, Utc};
use druid::kurbo::Insets;
use druid::widget::{Button, Flex, Label, Padding, Scroll, Widget, WidgetExt};
use druid::Color;
use druid::DelegateCtx;
use druid::WindowId;
use druid::{theme, Command};
use druid::{AppDelegate, AppLauncher, Data, Env, Event, Lens, WindowDesc};

use clock_widget::{Clock, ClockConfigData};

use crate::clock_widget;
use crate::parse_timeanddate_dot_com::{CityData, TimeData};

static mut CITY_COUNT: usize = 0;

fn city_count() -> usize {
    unsafe { CITY_COUNT }
}

fn set_city_count(new_value: usize) {
    unsafe {
        CITY_COUNT = new_value;
    }
}

///                              _______________________
///              ,~~~~,         /                       \    
///             //(??)\\ -=====( Oh Druid Please Show Us )
///             ~ _)(_ ~        \_______________________/
///            //(0 0)\\
///            \| ).( |/
///             @//^\\@
///              \| |/
///            `~~  ~~'
pub fn show(time_data: HashMap<String, TimeData>, test_mode: bool) {
    //Show one-by-one the time-data-items...
    for (k, d) in time_data {
        show_partition((k, &d), test_mode);
    }
}

fn show_partition((k, d): (String, &TimeData), test_mode: bool) {
    set_city_count(d.city_times.len());
    println!("Number of cities = {}", city_count());
    AppLauncher::with_window(WindowDesc::new(ui_builder).window_size((1500.0, 900.0)))
        .delegate(UiModel {
            test_mode,
            ..Default::default()
        })
        .launch((k, d).into())
        .unwrap()
}

fn ui_builder() -> impl Widget<UiModel> {
    //the top-most container; contains all rows...
    let mut col = Flex::column();
    let mut r = Flex::row();

    r.add_child(
        Label::new(move |model: &UiModel, _: &Env| format!("City Times for {}", model.title)),
        1.0,
    );
    col.add_child(Padding::new(Insets::from((20.0, 20.0, 0.0, 50.0)), r), 1.0);

    let inc = druid::widget::Button::sized(
        "Next City",
        |_, model: &mut UiModel, _| {
            model.next();
        },
        120.0,
        40.0,
    );

    let dec = Button::sized(
        "Previous City",
        |_, model: &mut UiModel, _| {
            model.previous();
        },
        120.0,
        40.0,
    );

    let reset = Button::sized(
        "Reset",
        |_, model: &mut UiModel, _| model.row_num = 0,
        120.0,
        40.0,
    );
    let mut r = Flex::row();

    r.add_child(dec.padding((0.0, 0.0, 20.0, 0.0)), 1.0);
    r.add_child(inc.padding((0.0, 0.0, 20.0, 0.0)), 1.0);
    r.add_child(reset.padding((0.0, 0.0, 20.0, 0.0)), 1.0);
    col.add_child(r.padding((20.0, 20.0, 0.0, 20.0)), 1.0);

    let mut r = Flex::row();
    r.add_child(
        Clock::new(
            move |model: &UiModel| create_clock_config_data(&model, model.up_previous()),
            250.0,
            true,
        ),
        1.0,
    );
    r.add_child(
        Clock::new(
            move |model: &UiModel| create_clock_config_data(&model, model.row_num),
            400.0,
            false,
        ),
        1.0,
    );
    r.add_child(
        Clock::new(
            move |model: &UiModel| create_clock_config_data(&model, model.up_next()),
            250.0,
            false,
        ),
        1.0,
    );

    col.add_child(r.padding((20.0, 20.0, 0.0, 50.0)), 1.0);

    //Now make one row containing two times three columns: details buttons, city-names and the times...
    r = Flex::row();
    let mut c_url = Flex::column();
    let mut c_name = Flex::column();
    let mut c_time = Flex::column();
    let mut c2_url = Flex::column();
    let mut c2_name = Flex::column();
    let mut c2_time = Flex::column();

    for i in 0..city_count() {
        let details = Button::new(format!("{}", '➚'), move |_, model: &mut UiModel, _| {
            Cmd::new("xdg-open")
                .arg(&model.cities[i].url.as_str())
                .output()
                .expect("Could not launch web-browser");
        })
        .padding((10.0, 0.0, 0.0, 0.0));

        let city_name =
            Label::new(move |model: &UiModel, _env: &_| format!("{}: ", model.cities[i].name,));

        let city_date = Label::new(move |model: &UiModel, _env: &_| {
            format!(
                "{} {}",
                DateTime::<FixedOffset>::from(Utc::now())
                    .with_timezone(&model.cities[i].utc_offset.get())
                    .format("%a, %Y-%m-%d %H:%M:%S %z"),
                if model.cities[i].is_dls { "- DST" } else { "" },
            )
        })
        .padding((0.0, 0.0, 150.0, 0.0))
        .env_scope(|env| {
            env.set(theme::LABEL_COLOR, Color::rgb8(255, 100, 250));
        });

        if i % 2 == 0 {
            c_url.add_child(details, 1.0);
            c_name.add_child(city_name, 1.0);
            c_time.add_child(city_date, 1.0);
        } else {
            c2_url.add_child(details, 1.0);
            c2_name.add_child(city_name, 1.0);
            c2_time.add_child(city_date, 1.0);
        }
    }
    r.add_child(c_url, 1.0);
    r.add_child(c_name, 5.0);
    r.add_child(c_time, 5.0);
    r.add_child(c2_url, 1.0);
    r.add_child(c2_name, 5.0);
    r.add_child(c2_time, 5.0);

    col.add_child(r, 1.0);
    let scroll = Scroll::new(col);
    scroll
}

impl AppDelegate<UiModel> for UiModel {
    fn event(
        &mut self,
        event: Event,
        _data: &mut UiModel,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) -> Option<Event> {
        match event {
            Event::AnimFrame(a) => println!("Anim Frame: {}", a),
            _ => {}
        }
        Some(event)
    }

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

    fn window_removed(
        &mut self,
        id: WindowId,
        _data: &mut UiModel,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        println!("Window Closed: {:?}", id);
    }
}

fn create_clock_config_data(model: &UiModel, row_num: usize) -> ClockConfigData {
    ClockConfigData::new(
        Some(model.cities[row_num].name.to_string()),
        model.cities[row_num].utc_offset.get(),
        Some(String::from("%A: %_d %B Day %_j / Week %_W")),
        Some(
            [
                Color::rgb8(
                    (100 + row_num * 30 % 255) as u8,
                    (40 + row_num * 30 % 100) as u8,
                    (100 + row_num * 10 % 155) as u8,
                ),
                Color::rgb8(
                    (120 + row_num * 15 % 125) as u8,
                    (0 + row_num * 7 % 100) as u8,
                    (120 + row_num * 40 % 255) as u8,
                ),
                Color::rgb8(
                    (100 + row_num * 80 % 125) as u8,
                    (0 + row_num * 15 % 200) as u8,
                    (100 + row_num * 10 % 125) as u8,
                ),
            ]
            .into(),
        ),
    )
}

#[derive(Debug, Clone, Default, Lens)]
struct UiModel {
    t: f64,
    title: String,
    ///The Set of City-data
    cities: Vec<CityData>,

    ///Keeps state of selected City using the three buttons
    row_num: usize,
    rendered_row: usize,
    test_mode: bool,
}

impl Data for UiModel {
    fn same(&self, _: &Self) -> bool {
        true
    }
}

impl From<(String, &TimeData)> for UiModel {
    fn from((title, data): (String, &TimeData)) -> UiModel {
        UiModel {
            title: title,
            row_num: data.city_times.len() / 10,
            cities: data.city_times.iter().cloned().collect(),
            ..Default::default()
        }
    }
}

impl UiModel {
    fn next(&mut self) {
        self.row_num = self.up_next();
    }
    fn up_next(&self) -> usize {
        if self.row_num < city_count() - 1 {
            self.row_num + 1
        } else {
            0
        }
    }
    fn previous(&mut self) {
        self.row_num = self.up_previous();
    }

    fn up_previous(&self) -> usize {
        if self.row_num > 0 {
            self.row_num - 1
        } else {
            city_count() - 1
        }
    }
}
