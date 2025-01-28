use std::io::Write;

pub fn code_gen(exports: &[Export], dll_name: &str) -> std::io::Result<Box<[u8]>> {
    let mut w = vec![];
    // prelude
    writeln!(
        &mut w,
        indoc::indoc! {"
        use windows::core::{{s, PCSTR}};
        use windows::Win32::Foundation::FARPROC;
        use windows::Win32::System::LibraryLoader::{{GetProcAddress, LoadLibraryA}};
        use windows::Win32::System::SystemInformation::GetSystemDirectoryA;

        macro_rules! shim {{
            ($idx:literal,$func:ident) => {{
                #[no_mangle]
                #[naked]
                pub unsafe extern \"C\" fn $func() {{
                    std::arch::naked_asm!(
                        \"jmp [{{funcs}}+{{}}]\",
                        const $idx * std::mem::size_of::<FARPROC>(),
                        funcs = sym FUNCS
                    )
                }}
            }};
        }}

        fn get_system_dir() -> Vec<u8> {{
            let mut buf = vec![0; 200];
            let length = unsafe {{ GetSystemDirectoryA(Some(&mut buf)) }};
            buf.truncate(length as usize);
            buf
        }}
    "}
    )?;
    // actual DLL specific code
    writeln!(
        &mut w,
        "static mut FUNCS: [FARPROC; {0}] = [None; {0}];",
        exports.len(),
    )?;

    for (i, ex) in exports.iter().enumerate() {
        writeln!(&mut w, "shim!({}, {});", i, ex.name)?;
    }

    writeln!(&mut w, "#[ctor::ctor]")?;
    writeln!(&mut w, "unsafe fn ctor() {{")?;
    writeln!(&mut w, "    let path = format!(\"{{}}\\\\{}\\0\", std::str::from_utf8(&get_system_dir()).unwrap());", dll_name)?;
    writeln!(
        &mut w,
        "    let handle = LoadLibraryA(PCSTR::from_raw(path.as_ptr())).unwrap();"
    )?;
    for (i, ex) in exports.iter().enumerate() {
        writeln!(
            &mut w,
            "    FUNCS[{}] = GetProcAddress(handle, s!(\"{}\"));",
            i, ex.name
        )?;
    }
    writeln!(&mut w, "}}")?;

    Ok(w.into_boxed_slice())
}

pub fn parse_exports(pe: &goblin::pe::PE) -> Option<Box<[Export]>> {
    let export_data = pe.export_data.as_ref()?;

    let mut exports = pe
        .exports
        .iter()
        .enumerate()
        .map(|(i, ex)| Export {
            // TODO handle this gracufully
            name: ex.name.unwrap().into(),
            // the export table returns the ordinal - the PE base ordinal
            ordinal: export_data.export_ordinal_table[i] as u32
                + export_data.export_directory_table.ordinal_base,
        })
        .collect::<Box<[_]>>();
    exports.sort();

    Some(exports)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Export {
    name: Box<str>,
    ordinal: u32,
}
