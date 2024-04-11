mod pretty_log;

fn main() -> anyhow::Result<()> {
    pretty_log::init();

    kers::App::new()?
        .settings(kers::AppSettings {
            name: "Test App",
            version: (0, 1, 0),
        })
        .run()
}
