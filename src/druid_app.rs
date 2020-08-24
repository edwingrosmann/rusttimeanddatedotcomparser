use chrono::{DateTime, FixedOffset, Utc};
use clock_widget::{Clock, ClockConfigData};
use druid::kurbo::Insets;
use druid::theme;
use druid::widget::{Button, Flex, Label, Padding, Scroll, Widget, WidgetExt};
use druid::Color;
use druid::DelegateCtx;
use druid::WindowId;
use druid::{AppDelegate, AppLauncher, Data, Env, Event, Lens, WindowDesc};
use std::collections::HashMap;
use std::process::Command as Cmd;

use crate::clock_widget;
use crate::druid_ui;
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
pub fn show(time_data: HashMap<String, TimeData>) {
    let mut windows = Vec::new();
    for (k, d) in time_data {
        windows.push(WindowDesc::new(druid_ui::ui_builder).window_size((1500.0, 900.0)))
    }

    let mut app = AppLauncher::<druid_ui::UiModel> {
        windows,
        env_setup: None,
        delegate: None,
    };
}
