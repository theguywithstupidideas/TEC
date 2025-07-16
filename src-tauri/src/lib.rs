use chrono::{NaiveDate, NaiveDateTime};
#[cfg(windows)]
use is_elevated::is_elevated;
#[cfg(windows)]
use std::ffi::{c_void, OsStr};
use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::ptr::null;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::core::PWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::ERROR_MEMBER_IN_ALIAS;
#[cfg(windows)]
use windows::Win32::Foundation::ERROR_SUCCESS;
#[cfg(windows)]
use windows::Win32::NetworkManagement::NetManagement::{
    NERR_Success, NetApiBufferFree, NetLocalGroupAddMembers, NetUserEnum, NetUserGetInfo,
    NetUserSetInfo, FILTER_NORMAL_ACCOUNT, LOCALGROUP_MEMBERS_INFO_3, UF_PASSWORD_EXPIRED,
    USER_INFO_0, USER_INFO_1008,
};
#[cfg(windows)]
use windows::Win32::NetworkManagement::NetManagement::{
    NetUserAdd, UF_NORMAL_ACCOUNT, USER_ACCOUNT_FLAGS, USER_INFO_2, USER_PRIV_USER,
};

#[cfg(windows)]
#[tauri::command]
fn create_user(
    username: String,
    password: Option<String>,
    full_name: Option<String>,
    expiration: Option<String>,
) -> Result<(), String> {
    let mut user = USER_INFO_2::default();

    // Keep these vectors alive for the entire function
    let username_vec: Vec<u16> = OsStr::new(&username).encode_wide().chain(Some(0)).collect();
    let password_vec: Vec<u16> = OsStr::new(password.as_deref().unwrap_or(""))
        .encode_wide()
        .chain(Some(0))
        .collect();
    let full_name_vec: Vec<u16> = OsStr::new(full_name.as_deref().unwrap_or(&username))
        .encode_wide()
        .chain(Some(0))
        .collect();

    user.usri2_name = PWSTR(username_vec.as_ptr() as *mut _);
    user.usri2_password = PWSTR(password_vec.as_ptr() as *mut _);
    user.usri2_full_name = PWSTR(full_name_vec.as_ptr() as *mut _);
    user.usri2_priv = USER_PRIV_USER;
    user.usri2_flags = USER_ACCOUNT_FLAGS(UF_NORMAL_ACCOUNT); // Remove UF_PASSWORD_EXPIRED for now

    // Set expiration time
    user.usri2_acct_expires = match expiration.as_deref().and_then(parse_exp) {
        Some(time) => {
            let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
            if duration.as_secs() > u32::MAX as u64 {
                u32::MAX // fallback to never expire
            } else {
                duration.as_secs() as u32
            }
        }
        None => u32::MAX,
    };

    unsafe {
        let mut error_param = 0u32;
        let status = NetUserAdd(
            None,
            2,
            &user as *const _ as *const u8,
            Option::from(&mut error_param as *mut u32),
        );

        if status == ERROR_SUCCESS.0 {
            println!("✓ Successfully created user: {}", username);
            group_add(&username, "Users")?;
            set_pass_exp(username)?;
            Ok(())
        } else {
            eprintln!(
                "⛌ Failed to create user. Error code: {}, Parameter: {}",
                status, error_param
            );
            Err(format!(
                "Failed to create user. Error code: {}, Parameter: {}",
                status, error_param
            ))
        }
    }
}

#[cfg(windows)]
#[tauri::command]
fn check_admin() -> bool {
    if !is_elevated() {
        false
    } else {
        true
    }
}

#[tauri::command]
fn read_event(file_path: &Path) -> Result<Vec<String>, String> {
    const HEAD: &[u8] = b"EVTKT";
    const SECTOR_1: &[u8] = b"SC1";
    const SECTOR_2: &[u8] = b"SC2";
    const END: &[u8] = b"END";
    const EOF: &[u8] = b"EOF";

    let file = File::open(file_path);
    let mut buffer: Vec<u8> = Vec::new();
    let mut offset: u64 = 0;
    let mut data_list: Vec<String> = vec![];
    buffer.resize(5, 0);
    match file {
        Ok(mut file) => {
            file.read_exact(&mut buffer).map_err(|e| e.to_string())?;
            if buffer != HEAD {
                return Err(format!(
                    "Header Mismatch: {}",
                    String::from_utf8_lossy(&buffer).to_string()
                ));
            }

            offset += (HEAD.len() + SECTOR_1.len()) as u64;
            buffer.resize(SECTOR_1.len(), 0);

            match read_sector(&file, &mut offset, &mut buffer, SECTOR_1) {
                Ok(data) => {
                    data_list.push(data);
                }

                Err(e) => return Err(format!("Error reading from file: {}", e)),
            }

            offset += (END.len() + SECTOR_2.len()) as u64;
            buffer.resize(SECTOR_2.len(), 0);

            match read_sector(&file, &mut offset, &mut buffer, SECTOR_2) {
                Ok(data) => {
                    data_list.push(data);
                }

                Err(e) => return Err(format!("Error reading from file: {}", e)),
            }

            match NaiveDate::parse_from_str(&data_list[1], "%m/%d/%Y") {
                Ok(..) => {  }
                Err(e) => return Err(format!("Error parsing date: {}", e)),
            }

            offset += (END.len() + EOF.len()) as u64;
            buffer.resize(EOF.len(), 0);

            seek_read(file, offset, &mut buffer).map_err(|e| e.to_string())?;

            if buffer != EOF {
                return Err(format!(
                    "EOF Mismatch: {}",
                    String::from_utf8_lossy(&buffer).to_string()
                ));
            }

            Ok(data_list)
        }
        Err(error) => Err(format!("Error: {}", error)),
    }
}

#[cfg(windows)]
#[tauri::command]
fn clean_up() -> Result<Vec<String>, String> {
    let mut usernames: Vec<String> = Vec::new();
    unsafe {
        let mut buffer: *mut c_void = ptr::null_mut();
        let mut entries_read = 0;
        let mut total_entries = 0;

        let status = NetUserEnum(
            None,
            0,
            FILTER_NORMAL_ACCOUNT,
            &mut buffer as *mut _ as *mut _,
            u32::MAX,
            &mut entries_read,
            &mut total_entries,
            Some(ptr::null_mut()),
        );

        if status != ERROR_SUCCESS.0 {
            return Err(format!("NetUserEnum failed: {}", status));
        }

        let users: *mut USER_INFO_0 = buffer as *mut USER_INFO_0;
        let user_slice = std::slice::from_raw_parts(users, entries_read as usize);

        for user in user_slice {
            let username = PCWSTR(user.usri0_name.0).to_string().unwrap_or_default();

            let mut user_info_buf: *mut c_void = ptr::null_mut();
            let status = NetUserGetInfo(
                None,
                user.usri0_name,
                2,
                &mut user_info_buf as *mut _ as *mut *mut u8,
            );
            if status == ERROR_SUCCESS.0 {
                let user_info: &USER_INFO_2 = &*(user_info_buf as *const USER_INFO_2);
                let expires = user_info.usri2_acct_expires;

                if expires != u32::MAX {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if expires as u64 <= now {
                        usernames.push(username.to_string());
                    }
                }

                NetApiBufferFree(Some(user_info_buf));
            }
        }

        NetApiBufferFree(Some(buffer));
    }

    Ok(usernames)
}

fn parse_exp(date: &str) -> Option<SystemTime> {
    let date = NaiveDate::parse_from_str(date, "%m/%d/%Y").ok()?;
    let datetime = NaiveDateTime::new(date, chrono::NaiveTime::from_hms_opt(0, 0, 0)?);
    let timestamp = datetime.and_utc().timestamp();

    if timestamp < 0 {
        None
    } else {
        Some(UNIX_EPOCH + Duration::from_secs(timestamp as u64))
    }
}

fn seek_read(mut reader: impl Read + Seek, offset: u64, buf: &mut [u8]) -> io::Result<()> {
    reader.seek(SeekFrom::Start(offset))?;
    reader.read_exact(buf)?;
    Ok(())
}

#[cfg(windows)]
fn group_add(username: &str, group_name: &str) -> Result<(), String> {
    let username_wide: Vec<u16> = OsStr::new(username).encode_wide().chain(Some(0)).collect();
    let group_name_wide: Vec<u16> = OsStr::new(group_name)
        .encode_wide()
        .chain(Some(0))
        .collect();

    let member = LOCALGROUP_MEMBERS_INFO_3 {
        lgrmi3_domainandname: PWSTR(username_wide.as_ptr() as *mut _),
    };

    unsafe {
        let status = NetLocalGroupAddMembers(
            None,
            PWSTR(group_name_wide.as_ptr() as *mut _),
            3,
            &member as *const _ as *const u8,
            1,
        );

        if status == ERROR_SUCCESS.0 || status == ERROR_MEMBER_IN_ALIAS.0 {
            println!(
                "✓ User '{}' added to local group '{}'",
                username, group_name
            );
            Ok(())
        } else {
            Err(format!(
                "⛌ Failed to add user to local group '{}'. Error code: {}",
                group_name, status
            ))
        }
    }
}

#[cfg(windows)]
fn set_pass_exp(username: String) -> Result<String, String> {
    unsafe {
        // Set UF_PASSWORD_EXPIRED flag (0x800000)
        let mut user_info = USER_INFO_1008 {
            usri1008_flags: UF_PASSWORD_EXPIRED,
        };

        // Call NetUserSetInfo with level 1008
        let result = NetUserSetInfo(
            PWSTR::null(), // local machine
            PCWSTR::from_raw(
                username
                    .encode_utf16()
                    .chain(Some(0))
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            1008,
            &mut user_info as *mut _ as *mut _,
            Some(ptr::null_mut()),
        );

        if result == NERR_Success {
            Ok("Successfully set UF_PASSWORD_EXPIRED flag.".to_string())
        } else {
            Err("An error has occurred.".to_string())
        }
    }
}

fn read_sector(
    file: &File,
    offset: &mut u64,
    buffer: &mut Vec<u8>,
    SECTOR: &[u8],
) -> Result<String, String> {
    const START: &[u8] = b"START";
    const END: &[u8] = b"END";
    const NULL_BYTE: &[u8] = b"\0";
    let mut data_len: u64 = 0;
    let mut res: String = String::new();

    seek_read(file, *offset, buffer).map_err(|e| e.to_string())?;
    if buffer != SECTOR {
        return Err(format!(
            "Failed to read sector. {:?} {}",
            String::from_utf8_lossy(SECTOR),
            offset
        ));
    }

    *offset += (SECTOR.len() + START.len()) as u64;
    buffer.resize(START.len(), 0);
    seek_read(file, *offset, buffer).map_err(|e| e.to_string())?;

    if buffer != START {
        return Err(format!(
            "Failed to read sector. {} {}",
            offset,
            String::from_utf8_lossy(&buffer).to_string()
        ));
    }

    *offset += START.len() as u64;

    buffer.resize(1, 0);

    seek_read(file, *offset, buffer).map_err(|e| e.to_string())?;

    while buffer == NULL_BYTE {
        seek_read(file, *offset, buffer).map_err(|e| e.to_string())?;
        *offset += 1;
        data_len += 1;
    }

    *offset -= 1;
    data_len -= 1;
    buffer.resize(data_len as usize, 0);
    seek_read(file, *offset, buffer).map_err(|e| e.to_string())?;

    res = String::from_utf8_lossy(&buffer).to_string();

    *offset += data_len + END.len() as u64;
    buffer.resize(END.len(), 0);
    seek_read(file, *offset, buffer).map_err(|e| e.to_string())?;

    if buffer != END {
        return Err(format!(
            "Failed to read sector. {} {}",
            offset,
            String::from_utf8_lossy(&buffer).to_string()
        ));
    }

    Ok(res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            #[cfg(windows)]
            check_admin,
            #[cfg(windows)]
            create_user,
            read_event,
            #[cfg(windows)]
            clean_up
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
