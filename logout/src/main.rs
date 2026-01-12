use relm4::prelude::*;

use dwsh_logout::app::DwshLogout;

fn main() {
    let app = RelmApp::new("io.ckgxrg.dwsh.logout");
    relm4::set_global_css(include_str!("../assets/style.css"));
    app.run::<DwshLogout>(());
}
