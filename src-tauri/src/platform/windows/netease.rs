use std::{cell::RefCell, env, ffi::c_void, fs, mem::size_of, path::PathBuf};

use serde_json::Value;
use windows::{
    core::{Error, Owned},
    Win32::{
        Foundation::{ERROR_NO_MORE_FILES, HANDLE, STILL_ACTIVE},
        System::{
            Diagnostics::{
                Debug::ReadProcessMemory,
                ToolHelp::{
                    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW,
                    Process32NextW, MODULEENTRY32W, PROCESSENTRY32W, TH32CS_SNAPMODULE,
                    TH32CS_SNAPPROCESS,
                },
            },
            Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READ},
            Threading::{
                GetExitCodeProcess, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
            },
        },
    },
};

use super::utf16z_to_string;

pub(super) const PROCESS_NAME: &str = "cloudmusic.exe";
const MODULE_NAME: &str = "cloudmusic.dll";
const PLAYING_LIST_PATH: &str = "Netease/CloudMusic/WebData/file/playingList";
const SONG_INFO_OFFSET: usize = 0x50;
const SONG_KEY_OFFSET: usize = 0x10;
const WILDCARD: i16 = -1;

// These RIP-relative instructions reference the NetEase 3.x playback clock and player.
const SCHEDULE_SIGNATURE: &[i16] = &[
    0x66, 0x0F, 0x2E, 0x0D, WILDCARD, WILDCARD, WILDCARD, WILDCARD, 0x7A, WILDCARD, 0x75, WILDCARD,
    0x66, 0x0F, 0x2E, 0x15,
];
const PLAYER_SIGNATURE: &[i16] = &[
    0x48, 0x8D, 0x0D, WILDCARD, WILDCARD, WILDCARD, WILDCARD, 0xE8, WILDCARD, WILDCARD, WILDCARD,
    WILDCARD, 0x48, 0x8D, 0x0D, WILDCARD, WILDCARD, WILDCARD, WILDCARD, 0xE8, WILDCARD, WILDCARD,
    WILDCARD, WILDCARD, 0x90, 0x48, 0x8D, 0x0D, WILDCARD, WILDCARD, WILDCARD, WILDCARD, 0xE8,
    WILDCARD, WILDCARD, WILDCARD, WILDCARD, 0x48, 0x8D, 0x05, WILDCARD, WILDCARD, WILDCARD,
    WILDCARD, 0x48, 0x8D, 0xA5, WILDCARD, WILDCARD, WILDCARD, WILDCARD, 0x5F, 0x5D, 0xC3, 0xCC,
    0xCC, 0xCC, 0xCC, 0xCC, 0x48, 0x89, 0x4C, 0x24, WILDCARD, 0x55, 0x57, 0x48, 0x81, 0xEC,
    WILDCARD, WILDCARD, WILDCARD, WILDCARD, 0x48, 0x8D, 0x6C, 0x24, WILDCARD, 0x48, 0x8D, 0x7C,
    0x24,
];
const SCHEDULE_DISPLACEMENT_OFFSET: usize = 4;
const PLAYER_DISPLACEMENT_OFFSET: usize = 3;
const MAX_POSITION_SECONDS: f64 = 86_400.0;

thread_local! {
    static PLAYBACK: RefCell<Option<Playback>> = const { RefCell::new(None) };
}

pub(super) fn read_timeline(title: &str) -> Result<(i64, i64), String> {
    PLAYBACK.with(|playback| {
        let mut playback = playback.borrow_mut();
        if let Some(reader) = playback.as_ref() {
            if reader.is_running()? {
                return reader.read_timeline(title);
            }
        }

        playback.insert(Playback::open()?).read_timeline(title)
    })
}

struct Playback {
    process: Owned<HANDLE>,
    schedule_address: usize,
    player_address: usize,
    playing_list_path: PathBuf,
}

impl Playback {
    fn open() -> Result<Self, String> {
        let pid = main_process_id()?;
        let process = unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)
                .map(|handle| Owned::new(handle))
        }
        .map_err(|error| format!("打开网易云主进程失败：{error}"))?;
        let (base, size) = music_module(pid)?;
        let (schedule_address, player_address) = find_timeline_addresses(*process, base, size)?;
        let playing_list_path = PathBuf::from(
            env::var_os("LOCALAPPDATA").ok_or_else(|| "找不到 LOCALAPPDATA。".to_string())?,
        )
        .join(PLAYING_LIST_PATH);
        Ok(Self {
            process,
            schedule_address,
            player_address,
            playing_list_path,
        })
    }

    fn is_running(&self) -> Result<bool, String> {
        let mut exit_code = 0;
        unsafe { GetExitCodeProcess(*self.process, &mut exit_code) }
            .map_err(|error| format!("读取网易云进程状态失败：{error}"))?;
        Ok(exit_code == STILL_ACTIVE.0 as u32)
    }

    fn song_id(&self) -> Result<String, String> {
        const INLINE_CAPACITY: usize = 15;
        const MAX_SONG_KEY_BYTES: usize = 128;

        let song_info = u64::from_le_bytes(read_bytes(
            *self.process,
            self.player_address + SONG_INFO_OFFSET,
        )?) as usize;
        if song_info == 0 {
            return Err("网易云主进程没有当前曲目。".into());
        }

        // MSVC's 64-bit std::string stores 16 inline bytes, then the length and capacity.
        let layout = read_bytes::<32>(*self.process, song_info + SONG_KEY_OFFSET)?;
        let storage = &layout[..16];
        let length = u64::from_le_bytes(layout[16..24].try_into().unwrap());
        let capacity = u64::from_le_bytes(layout[24..32].try_into().unwrap());
        if !(1..=MAX_SONG_KEY_BYTES as u64).contains(&length) || length > capacity {
            return Err("网易云当前曲目的 ID 长度无效。".into());
        }
        let length = length as usize;

        let mut heap_bytes = [0; MAX_SONG_KEY_BYTES];
        let bytes = if capacity > INLINE_CAPACITY as u64 {
            let address = u64::from_le_bytes(storage[..8].try_into().unwrap()) as usize;
            read_process(*self.process, address, &mut heap_bytes[..length])?;
            &heap_bytes[..length]
        } else {
            &storage[..length]
        };
        let key = std::str::from_utf8(bytes).map_err(|_| "网易云当前曲目的 ID 不是 UTF-8。")?;
        let id = key.split_once('_').map_or(key, |(id, _)| id);
        if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("网易云当前曲目的 ID 无效。".into());
        }
        Ok(id.into())
    }

    fn read_timeline(&self, title: &str) -> Result<(i64, i64), String> {
        let id = self.song_id()?;
        let seconds = f64::from_le_bytes(read_bytes(*self.process, self.schedule_address)?);
        if id != self.song_id()? {
            return Err("读取网易云时间轴时曲目发生了变化。".into());
        }
        if !(0.0..MAX_POSITION_SECONDS).contains(&seconds) {
            return Err("网易云当前曲目的播放进度无效。".into());
        }
        let duration = self.duration(&id, title)?;
        Ok(((seconds * 1_000.0).round() as i64, duration))
    }

    fn duration(&self, id: &str, title: &str) -> Result<i64, String> {
        let data = fs::read(&self.playing_list_path)
            .map_err(|error| format!("读取网易云播放列表缓存失败：{error}"))?;
        let queue: Value = serde_json::from_slice(&data)
            .map_err(|error| format!("解析网易云播放列表缓存失败：{error}"))?;
        let track = queue["list"]
            .as_array()
            .and_then(|list| {
                list.iter().find(|item| {
                    item["id"].as_str() == Some(id)
                        || item["id"]
                            .as_u64()
                            .is_some_and(|number| number.to_string() == id)
                })
            })
            .map(|item| &item["track"])
            .ok_or_else(|| "网易云播放列表缓存中没有当前曲目。".to_string())?;
        if track["name"].as_str().map(str::trim) != Some(title.trim()) {
            return Err("网易云播放列表中的当前曲目与 SMTC 曲名不一致。".into());
        }

        track["duration"]
            .as_f64()
            .map(f64::round)
            .filter(|duration| *duration > 0.0 && *duration < i64::MAX as f64)
            .map(|duration| duration as i64)
            .ok_or_else(|| "网易云播放列表缓存中的曲目时长无效。".into())
    }
}

fn main_process_id() -> Result<u32, String> {
    let snapshot =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map(|handle| Owned::new(handle)) }
            .map_err(|error| format!("获取进程快照失败：{error}"))?;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    unsafe { Process32FirstW(*snapshot, &mut entry) }
        .map_err(|error| format!("枚举进程失败：{error}"))?;

    let mut processes = Vec::new();
    loop {
        if utf16z_to_string(&entry.szExeFile).eq_ignore_ascii_case(PROCESS_NAME) {
            processes.push((entry.th32ProcessID, entry.th32ParentProcessID));
        }
        match unsafe { Process32NextW(*snapshot, &mut entry) } {
            Ok(()) => {}
            Err(error) if error.code() == ERROR_NO_MORE_FILES.to_hresult() => break,
            Err(error) => return Err(format!("枚举进程失败：{error}")),
        }
    }

    // The Chromium subprocesses run under another cloudmusic.exe.
    let mut roots = processes
        .iter()
        .filter(|(_, parent)| !processes.iter().any(|(pid, _)| pid == parent));
    match (roots.next(), roots.next()) {
        (Some(&(pid, _)), None) => Ok(pid),
        _ => Err("需要恰好一个网易云主进程。".into()),
    }
}

fn music_module(pid: u32) -> Result<(usize, usize), String> {
    let snapshot = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid).map(|handle| Owned::new(handle))
    }
    .map_err(|error| format!("获取网易云模块快照失败：{error}"))?;
    let mut entry = MODULEENTRY32W {
        dwSize: size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };
    unsafe { Module32FirstW(*snapshot, &mut entry) }
        .map_err(|error| format!("枚举网易云模块失败：{error}"))?;
    loop {
        if utf16z_to_string(&entry.szModule).eq_ignore_ascii_case(MODULE_NAME) {
            return Ok((entry.modBaseAddr as usize, entry.modBaseSize as usize));
        }
        match unsafe { Module32NextW(*snapshot, &mut entry) } {
            Ok(()) => {}
            Err(error) if error.code() == ERROR_NO_MORE_FILES.to_hresult() => {
                return Err(format!("网易云进程中找不到 {MODULE_NAME}。"));
            }
            Err(error) => return Err(format!("枚举网易云模块失败：{error}")),
        }
    }
}

fn find_timeline_addresses(
    process: HANDLE,
    base: usize,
    size: usize,
) -> Result<(usize, usize), String> {
    let end = base + size;
    let mut address = base;
    let mut schedule = None;
    let mut player = None;
    while address < end {
        let mut region = MEMORY_BASIC_INFORMATION::default();
        let queried = unsafe {
            VirtualQueryEx(
                process,
                Some(address as *const c_void),
                &mut region,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if queried == 0 {
            return Err(format!(
                "读取网易云模块内存区域失败：{}",
                Error::from_thread()
            ));
        }
        let region_end = (region.BaseAddress as usize + region.RegionSize).min(end);
        if region.State == MEM_COMMIT && region.Protect == PAGE_EXECUTE_READ {
            let mut bytes = vec![0; region_end - address];
            read_process(process, address, &mut bytes)?;

            if schedule.is_none() {
                schedule = find_rip_target(
                    &bytes,
                    address,
                    SCHEDULE_SIGNATURE,
                    SCHEDULE_DISPLACEMENT_OFFSET,
                );
            }
            if player.is_none() {
                player = find_rip_target(
                    &bytes,
                    address,
                    PLAYER_SIGNATURE,
                    PLAYER_DISPLACEMENT_OFFSET,
                );
            }
            if let (Some(schedule), Some(player)) = (schedule, player) {
                return Ok((schedule, player));
            }
        }
        address = region_end;
    }
    Err("当前网易云版本的播放时间轴内存签名未匹配。".into())
}

fn find_rip_target(
    bytes: &[u8],
    base: usize,
    signature: &[i16],
    displacement_offset: usize,
) -> Option<usize> {
    let offset = bytes.windows(signature.len()).position(|window| {
        window
            .iter()
            .zip(signature)
            .all(|(&byte, &expected)| expected == WILDCARD || i16::from(byte) == expected)
    })? + displacement_offset;
    let next_instruction = offset + size_of::<i32>();
    let displacement = i32::from_le_bytes(bytes[offset..next_instruction].try_into().unwrap());
    Some((base + next_instruction).wrapping_add_signed(displacement as isize))
}

fn read_bytes<const N: usize>(process: HANDLE, address: usize) -> Result<[u8; N], String> {
    let mut bytes = [0; N];
    read_process(process, address, &mut bytes)?;
    Ok(bytes)
}

fn read_process(process: HANDLE, address: usize, bytes: &mut [u8]) -> Result<(), String> {
    unsafe {
        ReadProcessMemory(
            process,
            address as *const c_void,
            bytes.as_mut_ptr().cast(),
            bytes.len(),
            None,
        )
    }
    .map_err(|error| format!("读取网易云进程内存失败：{error}"))
}
