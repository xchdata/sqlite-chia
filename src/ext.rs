// https://www.sqlite.org/loadext.html
// https://github.com/jgallagher/rusqlite/issues/524#issuecomment-507787350

use rusqlite::ffi;

#[no_mangle]
pub extern "C" fn sqlite3_extension_init(
    db_handle: *mut ffi::sqlite3,
    _pz_err_msg: &mut &mut std::os::raw::c_char,
    p_api: *mut ffi::sqlite3_api_routines,
) -> std::os::raw::c_int {
    // SQLITE_EXTENSION_INIT2 equivalent
    unsafe {
        ffi::sqlite3_api = p_api;
    }
    match init(db_handle) {
        Ok(()) => ffi::SQLITE_OK,
        Err(_e) => ffi::SQLITE_ERROR,
    }
}

fn init(db_handle: *mut ffi::sqlite3) -> anyhow::Result<()> {
    let db = unsafe { rusqlite::Connection::from_handle(db_handle)? };
    crate::setup(&db)?;
    Ok(())
}
