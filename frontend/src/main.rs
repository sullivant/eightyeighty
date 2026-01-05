slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = MainWindow::new()?;

    main_window.run()?;

    Ok(())
}