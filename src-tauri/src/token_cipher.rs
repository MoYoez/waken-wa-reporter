const DPAPI_MARKER: &str = "dpapi:";

#[cfg(target_os = "windows")]
mod windows_impl {
    use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    pub fn protect(plaintext: &str) -> Result<String, String> {
        if plaintext.is_empty() {
            return Ok(String::new());
        }
        let mut input = plaintext.as_bytes().to_vec();
        let in_blob = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let mut out_blob = CRYPT_INTEGER_BLOB::default();
        unsafe {
            CryptProtectData(
                &in_blob,
                PCWSTR::null(),
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out_blob,
            )
            .map_err(|error| format!("加密令牌失败：{error}"))?;
        }
        let encrypted = unsafe {
            std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec()
        };
        unsafe {
            let _ = LocalFree(Some(HLOCAL(out_blob.pbData as *mut _)));
        }
        Ok(BASE64_STANDARD.encode(encrypted))
    }

    pub fn unprotect(encoded: &str) -> Result<String, String> {
        if encoded.is_empty() {
            return Ok(String::new());
        }
        let mut input = BASE64_STANDARD
            .decode(encoded)
            .map_err(|error| format!("解析加密令牌失败：{error}"))?;
        let in_blob = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let mut out_blob = CRYPT_INTEGER_BLOB::default();
        unsafe {
            CryptUnprotectData(
                &in_blob,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out_blob,
            )
            .map_err(|error| format!("解密令牌失败：{error}"))?;
        }
        let decrypted = unsafe {
            std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec()
        };
        unsafe {
            let _ = LocalFree(Some(HLOCAL(out_blob.pbData as *mut _)));
        }
        String::from_utf8(decrypted).map_err(|error| format!("解密令牌编码无效：{error}"))
    }
}

#[cfg(target_os = "windows")]
pub fn encrypt_for_storage(plaintext: &str) -> String {
    if plaintext.is_empty() {
        return String::new();
    }
    match windows_impl::protect(plaintext) {
        Ok(encoded) => format!("{DPAPI_MARKER}{encoded}"),
        Err(error) => {
            eprintln!("令牌静态加密失败，将以明文存储：{error}");
            plaintext.to_string()
        }
    }
}

#[cfg(target_os = "windows")]
pub fn decrypt_from_storage(stored: &str) -> String {
    match stored.strip_prefix(DPAPI_MARKER) {
        Some(encoded) => match windows_impl::unprotect(encoded) {
            Ok(plaintext) => plaintext,
            Err(error) => {
                eprintln!("令牌静态解密失败：{error}");
                String::new()
            }
        },
        None => stored.to_string(),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn encrypt_for_storage(plaintext: &str) -> String {
    plaintext.to_string()
}

#[cfg(not(target_os = "windows"))]
pub fn decrypt_from_storage(stored: &str) -> String {
    stored
        .strip_prefix(DPAPI_MARKER)
        .map(str::to_string)
        .unwrap_or_else(|| stored.to_string())
}
