// All export APIs of libdlt are implemented here

use dlt_user::dlt_user;

/// DltReturnValue enumerations are taken from the DLT header.
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum DltReturnValue {
    DLT_RETURN_FILESZERR = -8,
    DLT_RETURN_LOGGING_DISABLED = -7,
    DLT_RETURN_USER_BUFFER_FULL = -6,
    DLT_RETURN_WRONG_PARAMETER = -5,
    DLT_RETURN_BUFFER_FULL = -4,
    DLT_RETURN_PIPE_FULL = -3,
    DLT_RETURN_PIPE_ERROR = -2,
    DLT_RETURN_ERROR = -1,
    DLT_RETURN_OK = 0,
    DLT_RETURN_TRUE = 1,
}

#[no_mangle]
pub extern "C" fn dlt_init() -> DltReturnValue {
    // force the creation of the singleton
    let _dlt_user = dlt_user();
    DltReturnValue::DLT_RETURN_OK
}

#[no_mangle]
pub extern "C" fn dlt_free() -> DltReturnValue {
    DltReturnValue::DLT_RETURN_OK
}
