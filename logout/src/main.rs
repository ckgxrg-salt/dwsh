use relm4::prelude::*;

use dwsh_logout::app::LogoutApp;

fn main() {
    let app = RelmApp::new("io.ckgxrg.dwsh.logout");
    app.run::<LogoutApp>(());
}
