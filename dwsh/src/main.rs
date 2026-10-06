use relm4::prelude::*;

use dwsh::dashboard::Dashboard;

fn main() {
    let app = RelmApp::new("io.ckgxrg.dwsh");

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    // You couln't find
    //
    // your CSS.
    // Fix css being overridden by something else
    relm4::set_global_css_with_priority(include_str!("../assets/style.css"), 1225);

    app.run::<Dashboard>(());
}
