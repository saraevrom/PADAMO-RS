use iced::{Application, Settings};

mod messages;
mod application;
mod tools;
mod custom_widgets;
mod nodes_interconnect;
//mod detectors;
mod builtin_nodes;
mod datetime_parser;
mod time_search;
mod popup_message;
mod double_entry_state;

fn main() -> iced::Result{
    application::Padamo::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}
