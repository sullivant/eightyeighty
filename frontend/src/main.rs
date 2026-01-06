slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    main_window.on_cb_show_settings(|| {
        let dialog = SettingsDialog::new().unwrap();
        dialog.show().unwrap();
    });

    main_window.on_cb_exit(|| {
        slint::quit_event_loop().unwrap();
    });

    main_window.show()?;
    slint::run_event_loop()?;

    Ok(())
}