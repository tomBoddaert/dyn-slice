use std::{ffi::OsStr, fs::read_dir, path::Path};

#[test]
// Make sure that the files in compile_fail_tests fail to compile
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();

    let dir_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("compile_fail_tests");
    let dir = read_dir(dir_path).unwrap();

    for entry_result in dir {
        let entry = entry_result.unwrap();
        if entry.file_type().unwrap().is_dir()
            || !matches!(entry.path().extension().and_then(OsStr::to_str), Some("rs"))
        {
            continue;
        }

        t.compile_fail(entry.path());
    }
}
