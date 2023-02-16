pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod kv_storage {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum WriteError {
        #[error("unknown")]
        Unknown,
    }

    #[derive(Error, Debug)]
    pub enum ReadError {
        #[error("unknown")]
        Unknown,
    }

    pub trait KvStorage {
        type WriteErrorType;
        type ReadErrorType;

        fn read(&self, key: &str) -> Result<String, Self::ReadErrorType>;
        fn write(&self, key: &str, value: &str) -> Result<(), Self::WriteErrorType>;
    }

    #[cfg(target_family = "wasm")]
    pub mod wasm_cookies_kv_storage {
        use crate::kv_storage;
        use core::convert::Infallible;
        use thiserror::Error;
        use wasm_cookies::cookies;

        #[derive(Default)]
        pub struct WasmCookiesKvStorage;

        impl kv_storage::KvStorage for WasmCookiesKvStorage {
            type ReadErrorType = WasmCookieReadError;
            type WriteErrorType = Infallible;

            fn read(&self, key: &str) -> Result<String, Self::ReadErrorType> {
                match wasm_cookies::get(key) {
                    Some(cookie) => cookie.map_err(Into::into),
                    None => Ok("".to_string()),
                }
            }

            fn write(&self, key: &str, value: &str) -> Result<(), Self::WriteErrorType> {
                let cookie_options = cookies::CookieOptions::default();
                wasm_cookies::set(key, value, &cookie_options);
                Ok(())
            }
        }

        #[derive(Error, Debug)]
        pub enum WasmCookieReadError {
            #[error("Error url decoding")]
            UrlDecodeError(#[from] wasm_cookies::FromUrlEncodingError),

            #[error(transparent)]
            Other(#[from] kv_storage::ReadError),
        }
    }

    #[cfg(any(target_os = "windows", target_os = "android"))]
    pub mod file_based_kv_storage {
        use crate::kv_storage;
        use std::fs;
        use std::path::PathBuf;

        const APP_NAME: &str = "PokeIpGo"; // TODO: get this programatically

        pub struct FileBasedKvStorage(PathBuf);

        impl Default for FileBasedKvStorage {
            fn default() -> Self {
                log::info!("path: {:?}", Self::get_roaming_path());
                FileBasedKvStorage(Self::get_roaming_path().into())
            }
        }

        impl FileBasedKvStorage {
            #[cfg(target_os = "windows")]
            fn get_roaming_path() -> PathBuf {
                const ROAMING_ENV: &str = "APPDATA";

                let mut path: PathBuf = std::env::var(ROAMING_ENV)
                    .expect("could not get roaming dir")
                    .into();

                path.push(APP_NAME);
                path
            }

            #[cfg(target_os = "android")]
            fn get_roaming_path() -> PathBuf {
                PathBuf::from("./store")
            }
        }

        impl kv_storage::KvStorage for FileBasedKvStorage {
            type WriteErrorType = std::io::Error;
            type ReadErrorType = std::io::Error;

            fn read(&self, key: &str) -> Result<String, Self::ReadErrorType> {
                fs::create_dir_all(&self.0)?;
                let path = self.0.with_file_name(key);
                fs::read_to_string(path).map_err(|e| {
                    log::warn!(
                        "Could not read path '{}', key '{key}': {e}",
                        self.0.display()
                    );
                    e
                })
            }

            fn write(&self, key: &str, value: &str) -> Result<(), Self::WriteErrorType> {
                fs::create_dir_all(&self.0)?;
                let path = self.0.with_file_name(key);
                fs::write(path, value)
            }
        }
    }
}
