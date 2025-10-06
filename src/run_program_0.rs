
pub fn run_program_0() {
    // Parse the ELF
    let module = MODULE_REQUEST
        .get_response()
        .unwrap()
        .modules()
        .iter()
        .find(|module| module.path() == USER_MODE_PROGRAM_0_PATH)

    let ptr = NonNull::new(slice_from_raw_parts_mut(
        module.addr(),
        module.size() as usize,
    ))
    .unwrap();
    // Safety: Limine gives us a valid pointer and len
    let elf_bytes = unsafe { ptr.as_ref() };
    let elf = ElfBytes::<AnyEndian>::minimal_parse(elf_bytes).expect("ELF should be valid");
}
