use winapi::um::dpapi::{CryptProtectData, CryptUnprotectData};
use winapi::um::wincrypt::DATA_BLOB;
use winapi::um::winbase::LocalFree;

pub fn protect_password(password: &str) -> Result<Vec<u8>, String> {
    let data = password.as_bytes();
    let mut input = DATA_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = DATA_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptProtectData(
            &mut input,
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut output,
        )
    };

    if ok == 0 {
        return Err(format!(
            "CryptProtectData failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let bytes = unsafe {
        let b = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData as _);
        b
    };

    Ok(bytes)
}

pub fn unprotect_password(encrypted: &[u8]) -> Result<String, String> {
    let mut input = DATA_BLOB {
        cbData: encrypted.len() as u32,
        pbData: encrypted.as_ptr() as *mut u8,
    };
    let mut output = DATA_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &mut input,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut output,
        )
    };

    if ok == 0 {
        return Err(format!(
            "CryptUnprotectData failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let result = unsafe {
        let b = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData as _);
        b
    };

    String::from_utf8(result).map_err(|e| format!("Invalid UTF-8: {}", e))
}
