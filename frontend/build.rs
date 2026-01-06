use std::{collections::HashMap, path::PathBuf, fs};

fn main() {
    let out = PathBuf::from("ui/vendor/lucide.slint");
    fs::create_dir_all("ui/vendor").unwrap();
    fs::copy(lucide_slint::get_slint_file_path(), &out).unwrap();

    println!("Copied lucide slint from {:?} to {:?}", lucide_slint::get_slint_file_path(), out);

    let library = HashMap::from([
        ("lucide".to_string(), out),
    ]);

    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);
 
    // Specify your Slint code entry here
    slint_build::compile_with_config("ui/main.slint", config).expect("Slint build failed");
}