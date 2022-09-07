fn init_logger() {
    use simplelog::*;

    let level = LevelFilter::Debug;
    let config = Config::default();
    let colors = if atty::is(atty::Stream::Stdout) { ColorChoice::Auto} else { ColorChoice::Never};

    TermLogger::init(level, config, TerminalMode::Stdout, colors).expect("could not init logger");
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_logger();
    log::info!("hej");
    log::debug!("hejsan");
}
