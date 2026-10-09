//! A small safe wrapper over the PDH performance counter API.

// The PDH C API needs `unsafe`; it is confined to this module.
#![allow(unsafe_code)]

use windows_sys::Win32::System::Performance::{
    PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE,
    PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA, PERF_REG_COUNTER_ENGLISH_NAMES,
    PERF_REG_COUNTER_NAME_STRINGS, PERF_REG_COUNTERSET_ENGLISH_NAME,
    PERF_REG_COUNTERSET_NAME_STRING, PdhAddCounterW, PdhAddEnglishCounterW, PdhCloseQuery,
    PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW, PerfEnumerateCounterSet,
    PerfQueryCounterSetRegistrationInfo, PerfRegInfoType,
};
use windows_sys::core::GUID;

/// A PDH query holding one wildcard counter.
#[derive(Debug)]
pub(crate) struct Query {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
}

// SAFETY: PDH handles aren't tied to the thread that created them; Calcine
// only uses a query from one thread at a time (behind a mutex).
unsafe impl Send for Query {}

impl Query {
    /// `path` uses English names, whatever the Windows language.
    pub(crate) fn open(path: &str) -> Result<Self, String> {
        Self::add(path, true)
    }

    /// `path` uses the names Windows shows, see [`translated_path`].
    pub(crate) fn open_translated(path: &str) -> Result<Self, String> {
        Self::add(path, false)
    }

    fn add(path: &str, english: bool) -> Result<Self, String> {
        let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut query: PDH_HQUERY = std::ptr::null_mut();
        // SAFETY: a null data source means live counters; `query` is a valid
        // out pointer.
        check(unsafe { PdhOpenQueryW(std::ptr::null(), 0, &raw mut query) })?;
        let mut counter: PDH_HCOUNTER = std::ptr::null_mut();
        // SAFETY: `query` is open and `path` is NUL-terminated UTF-16 that
        // outlives the call.
        let added = unsafe {
            if english {
                PdhAddEnglishCounterW(query, path.as_ptr(), 0, &raw mut counter)
            } else {
                PdhAddCounterW(query, path.as_ptr(), 0, &raw mut counter)
            }
        };
        if let Err(err) = check(added) {
            // SAFETY: `query` is open and closed only here.
            unsafe { PdhCloseQuery(query) };
            return Err(err);
        }
        Ok(Self { query, counter })
    }

    /// Collect a sample and return each instance's value.
    pub(crate) fn read(&self) -> Result<Vec<(String, f64)>, String> {
        // SAFETY: `self.query` stays open for `self`'s lifetime.
        check(unsafe { PdhCollectQueryData(self.query) })?;

        let mut size = 0_u32;
        let mut count = 0_u32;
        // SAFETY: a null buffer with size 0 asks for the required size.
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &raw mut size,
                &raw mut count,
                std::ptr::null_mut(),
            )
        };
        if status != PDH_MORE_DATA {
            // No instances (nothing uses the GPU or NPU): an empty sample.
            return Ok(Vec::new());
        }

        // The buffer holds the items followed by the strings they point to;
        // u64 storage keeps it aligned for the items.
        let mut buffer = vec![0_u64; (size as usize).div_ceil(8)];
        let items = buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        // SAFETY: `buffer` is at least `size` bytes and suitably aligned.
        check(unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &raw mut size,
                &raw mut count,
                items,
            )
        })?;

        let mut samples = Vec::with_capacity(count as usize);
        for index in 0..count as usize {
            // SAFETY: PDH wrote `count` items at the start of `buffer`.
            let item = unsafe { *items.add(index) };
            let status = item.FmtValue.CStatus;
            if status != PDH_CSTATUS_VALID_DATA && status != PDH_CSTATUS_NEW_DATA {
                continue;
            }
            // SAFETY: `szName` points to a NUL-terminated string inside `buffer`.
            let name = unsafe { wide_to_string(item.szName) };
            // SAFETY: PDH_FMT_DOUBLE fills the `doubleValue` member.
            let value = unsafe { item.FmtValue.Anonymous.doubleValue };
            samples.push((name, value));
        }
        Ok(samples)
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        // SAFETY: the query is open and dropped once.
        unsafe { PdhCloseQuery(self.query) };
    }
}

/// # Safety
///
/// `ptr` must be null or point to a NUL-terminated UTF-16 string.
unsafe fn wide_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    // SAFETY: guaranteed NUL-terminated by the caller.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: `len` u16s before the NUL are readable.
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
}

fn check(status: u32) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("performance counter error 0x{status:08X}"))
    }
}

/// The path of a counter in a counter set, both given by their English names,
/// written with the names Windows shows (`\Compteur Énergie(*)\Énergie`
/// on a French Windows). PDH can't add counter sets declared by a manifest,
/// like `Energy Meter`, by their English names on a non-English Windows.
pub(crate) fn translated_path(english_set: &str, english_counter: &str) -> Result<String, String> {
    for set in counter_sets()? {
        if registration_string(&set, PERF_REG_COUNTERSET_ENGLISH_NAME).as_deref()
            != Some(english_set)
        {
            continue;
        }
        let name = registration_string(&set, PERF_REG_COUNTERSET_NAME_STRING)
            .ok_or_else(|| format!("{english_set} has no name"))?;
        let id = counter_strings(&set, PERF_REG_COUNTER_ENGLISH_NAMES)
            .into_iter()
            .find(|(_, counter)| counter == english_counter)
            .map(|(id, _)| id)
            .ok_or_else(|| format!("{english_set} has no {english_counter} counter"))?;
        let counter = counter_strings(&set, PERF_REG_COUNTER_NAME_STRINGS)
            .into_iter()
            .find(|(other, _)| *other == id)
            .map(|(_, counter)| counter)
            .ok_or_else(|| format!("{english_set} has no name for {english_counter}"))?;
        return Ok(format!(r"\{name}(*)\{counter}"));
    }
    Err(format!("there is no {english_set} counter set"))
}

/// Every counter set registered with PerfLib (manifest-based ones).
fn counter_sets() -> Result<Vec<GUID>, String> {
    let mut count = 0_u32;
    // SAFETY: a null buffer with size 0 asks for the count.
    unsafe { PerfEnumerateCounterSet(std::ptr::null(), std::ptr::null_mut(), 0, &raw mut count) };
    let mut sets = vec![GUID::from_u128(0); count as usize];
    // SAFETY: `sets` holds `count` GUIDs.
    check(unsafe {
        PerfEnumerateCounterSet(std::ptr::null(), sets.as_mut_ptr(), count, &raw mut count)
    })?;
    sets.truncate(count as usize);
    Ok(sets)
}

/// A counter set's registration data, in the current UI language.
fn registration(set: &GUID, request: PerfRegInfoType) -> Option<Vec<u8>> {
    let mut size = 0_u32;
    // SAFETY: a null buffer with size 0 asks for the size.
    unsafe {
        PerfQueryCounterSetRegistrationInfo(
            std::ptr::null(),
            set,
            request,
            0,
            std::ptr::null_mut(),
            0,
            &raw mut size,
        )
    };
    if size == 0 {
        return None;
    }
    let mut buffer = vec![0_u8; size as usize];
    // SAFETY: `buffer` holds `size` bytes.
    let status = unsafe {
        PerfQueryCounterSetRegistrationInfo(
            std::ptr::null(),
            set,
            request,
            0,
            buffer.as_mut_ptr(),
            size,
            &raw mut size,
        )
    };
    (status == 0).then_some(buffer)
}

fn registration_string(set: &GUID, request: PerfRegInfoType) -> Option<String> {
    registration(set, request).map(|bytes| utf16_at(&bytes, 0))
}

/// `PERF_STRING_BUFFER_HEADER`, then one `(counter id, offset)` pair per
/// counter, then the strings the offsets point to.
fn counter_strings(set: &GUID, request: PerfRegInfoType) -> Vec<(u32, String)> {
    let Some(bytes) = registration(set, request) else {
        return Vec::new();
    };
    let u32_at = |offset: usize| {
        bytes
            .get(offset..offset + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let count = u32_at(4).unwrap_or(0) as usize;
    (0..count)
        .filter_map(|index| {
            let entry = 8 + index * 8;
            let id = u32_at(entry)?;
            let offset = u32_at(entry + 4)? as usize;
            Some((id, utf16_at(&bytes, offset)))
        })
        .collect()
}

/// The NUL-terminated UTF-16 string at `offset` in `bytes`.
fn utf16_at(bytes: &[u8], offset: usize) -> String {
    let units: Vec<u16> = bytes
        .get(offset..)
        .unwrap_or_default()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .take_while(|&c| c != 0)
        .collect();
    String::from_utf16_lossy(&units)
}
