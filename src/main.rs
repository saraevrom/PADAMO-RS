
mod messages;
mod application;
mod tools;
mod custom_widgets;
mod nodes_interconnect;
//mod detectors;
mod builtin_nodes;
// mod datetime_parser;
mod time_search;
mod popup_message;
mod assets;
mod transform_widget;

fn main() -> iced::Result{
    // application::Padamo::run(Settings {
    //     antialiasing: true,
    //     ..Settings::default()
    // })
    iced::application("PADAMO",application::Padamo::update, application::Padamo::view)
        .subscription(application::Padamo::subscription)
        .exit_on_close_request(true)
        .run()
}
