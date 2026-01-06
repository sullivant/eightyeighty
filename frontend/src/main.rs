slint::include_modules!();


fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    ui.global::<AppLogic>().on_cb_exit(|| {
        slint::quit_event_loop().unwrap();
    });

    ui.global::<AppLogic>().on_cb_show_settings(|| {
        println!("Showing settings...");
    });

    ui.show()?;
    slint::run_event_loop()?;

    Ok(())
}