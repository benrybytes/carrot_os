let user_mode_program_0_executable_file = env::var("CARGO_BIN_FILE_USER_MODE_PROGRAM_0").unwrap();
ensure_symlink(
    user_mode_program_0_executable_file,
    iso_dir.join("user_mode_program_0"),
)
.unwrap();
