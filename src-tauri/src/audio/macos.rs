//! System audio capture on macOS through Core Audio process taps.
//!
//! A tap is an object that mirrors the audio a set of processes sends to the
//! hardware. To read from it we attach it to a private aggregate device and run
//! an IO block on that device. This needs no virtual audio driver and asks the
//! user for the Audio Recording permission rather than Screen Recording.
//!
//! Requires macOS 14.4 or newer.

use std::ffi::{c_void, CStr};
use std::ptr::{null, NonNull};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{AllocAnyThread, Message};
use objc2_core_audio::{
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceIsStackedKey,
    kAudioAggregateDeviceMainSubDeviceKey, kAudioAggregateDeviceNameKey,
    kAudioAggregateDeviceSubDeviceListKey, kAudioAggregateDeviceTapAutoStartKey,
    kAudioAggregateDeviceTapListKey, kAudioAggregateDeviceUIDKey, kAudioDevicePropertyDeviceUID,
    kAudioHardwarePropertyDefaultOutputDevice, kAudioHardwarePropertyProcessObjectList,
    kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject,
    kAudioProcessPropertyBundleID, kAudioProcessPropertyIsRunningOutput, kAudioProcessPropertyPID,
    kAudioSubDeviceUIDKey, kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey,
    kAudioTapPropertyFormat, AudioDeviceCreateIOProcIDWithBlock, AudioDeviceDestroyIOProcID,
    AudioDeviceIOProcID, AudioDeviceStart, AudioDeviceStop, AudioHardwareCreateAggregateDevice,
    AudioHardwareCreateProcessTap, AudioHardwareDestroyAggregateDevice,
    AudioHardwareDestroyProcessTap, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
    AudioObjectID, AudioObjectPropertyAddress, AudioObjectPropertySelector, CATapDescription,
    CATapMuteBehavior,
};
use objc2_core_audio_types::{AudioBufferList, AudioStreamBasicDescription, AudioTimeStamp};
use objc2_core_foundation::{CFDictionary, CFRetained, CFString};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString, NSUUID};

use super::{AudioError, CaptureFormat, SampleSink, SourceInfo, SourceKind, SourceSelector};

/// Largest planar channel count we mix without allocating. Real output devices
/// are stereo; the tap mixes down for us anyway.
const MAX_PLANES: usize = 8;

const SYSTEM_OBJECT: AudioObjectID = kAudioObjectSystemObject as AudioObjectID;

/// Listing ourselves would let the user tap Marswind's own (silent) output.
const OWN_BUNDLE_ID: &str = "com.marswind.app";

pub fn list_sources() -> Result<Vec<SourceInfo>, AudioError> {
    let mut sources = vec![SourceInfo {
        id: "system".to_string(),
        name: "All system audio".to_string(),
        detail: Some("Everything this Mac plays".to_string()),
        kind: SourceKind::System,
        active: true,
    }];

    for object_id in process_object_ids()? {
        let bundle_id = cfstring_property(object_id, kAudioProcessPropertyBundleID)
            .filter(|value| !value.is_empty());
        let pid = fixed_property::<i32>(object_id, kAudioProcessPropertyPID).ok();
        let active = fixed_property::<u32>(object_id, kAudioProcessPropertyIsRunningOutput)
            .map(|v| v != 0)
            .unwrap_or(false);

        if !is_worth_listing(bundle_id.as_deref(), active) {
            continue;
        }

        let name = bundle_id
            .as_deref()
            .map(display_name_for_bundle)
            .or_else(|| pid.map(|p| format!("PID {p}")))
            .unwrap_or_else(|| format!("Audio object {object_id}"));

        sources.push(SourceInfo {
            id: format!("process:{object_id}"),
            name,
            detail: bundle_id,
            kind: SourceKind::Process,
            active,
        });
    }

    sources.sort_by(|a, b| {
        // System first, then processes currently making sound, then the rest.
        (
            a.kind == SourceKind::Process,
            !a.active,
            a.name.to_lowercase(),
        )
            .cmp(&(
                b.kind == SourceKind::Process,
                !b.active,
                b.name.to_lowercase(),
            ))
    });

    Ok(sources)
}

pub fn source_name(selector: &SourceSelector) -> Result<String, AudioError> {
    match selector {
        SourceSelector::System => Ok("All system audio".to_string()),
        SourceSelector::Process(object_id) => {
            let bundle_id = cfstring_property(*object_id, kAudioProcessPropertyBundleID);
            Ok(bundle_id
                .as_deref()
                .map(display_name_for_bundle)
                .unwrap_or_else(|| format!("Audio object {object_id}")))
        }
    }
}

/// The callback Core Audio drives the IO proc with: now, input, input time,
/// output, output time. Only the input pair is ever read here.
type IoBlock = RcBlock<
    dyn Fn(
        NonNull<AudioTimeStamp>,
        NonNull<AudioBufferList>,
        NonNull<AudioTimeStamp>,
        NonNull<AudioBufferList>,
        NonNull<AudioTimeStamp>,
    ),
>;

/// Live capture. Dropping this tears down the OS objects in the right order.
pub struct Capture {
    tap_id: AudioObjectID,
    aggregate_id: AudioObjectID,
    proc_id: AudioDeviceIOProcID,
    /// Core Audio copies the block, but the closure owns the sink, so the block
    /// must outlive the IO proc.
    _block: IoBlock,
}

// SAFETY: the contained handles are plain integers plus a block owned by this
// struct; Core Audio calls the block on its own IO thread regardless of which
// thread holds the handle.
unsafe impl Send for Capture {}

impl Capture {
    pub fn start(
        selector: &SourceSelector,
        sink: SampleSink,
    ) -> Result<(Self, CaptureFormat), AudioError> {
        let description = tap_description(selector)?;

        let mut tap_id: AudioObjectID = 0;
        // SAFETY: `description` is a live CATapDescription and `tap_id` is a
        // valid out pointer.
        let status = unsafe { AudioHardwareCreateProcessTap(Some(&description), &mut tap_id) };
        check(status, "AudioHardwareCreateProcessTap")?;

        // From here on every early return must release what was created.
        let result = Self::finish_start(&description, tap_id, sink);
        if result.is_err() {
            unsafe { AudioHardwareDestroyProcessTap(tap_id) };
        }
        result
    }

    fn finish_start(
        description: &CATapDescription,
        tap_id: AudioObjectID,
        sink: SampleSink,
    ) -> Result<(Self, CaptureFormat), AudioError> {
        let tap_uid = unsafe { description.UUID() }.UUIDString().to_string();
        let aggregate_id = create_aggregate_device(&tap_uid)?;

        let format = tap_format(tap_id).inspect_err(|_| {
            unsafe { AudioHardwareDestroyAggregateDevice(aggregate_id) };
        })?;

        let block = io_block(sink);
        let mut proc_id: AudioDeviceIOProcID = None;

        // SAFETY: `proc_id` is a valid out pointer, the aggregate device exists,
        // and the block outlives the IO proc because `Capture` owns it.
        let status = unsafe {
            AudioDeviceCreateIOProcIDWithBlock(
                NonNull::from(&mut proc_id),
                aggregate_id,
                None,
                RcBlock::as_ptr(&block),
            )
        };
        if let Err(e) = check(status, "AudioDeviceCreateIOProcIDWithBlock") {
            unsafe { AudioHardwareDestroyAggregateDevice(aggregate_id) };
            return Err(e);
        }

        let status = unsafe { AudioDeviceStart(aggregate_id, proc_id) };
        if let Err(e) = check(status, "AudioDeviceStart") {
            unsafe {
                AudioDeviceDestroyIOProcID(aggregate_id, proc_id);
                AudioHardwareDestroyAggregateDevice(aggregate_id);
            }
            return Err(e);
        }

        Ok((
            Self {
                tap_id,
                aggregate_id,
                proc_id,
                _block: block,
            },
            format,
        ))
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: teardown mirrors setup, and each handle is destroyed once.
        unsafe {
            AudioDeviceStop(self.aggregate_id, self.proc_id);
            AudioDeviceDestroyIOProcID(self.aggregate_id, self.proc_id);
            AudioHardwareDestroyAggregateDevice(self.aggregate_id);
            AudioHardwareDestroyProcessTap(self.tap_id);
        }
    }
}

fn tap_description(selector: &SourceSelector) -> Result<Retained<CATapDescription>, AudioError> {
    // SAFETY: standard Objective-C alloc/init, arrays are live for the call.
    let description = unsafe {
        match selector {
            SourceSelector::System => {
                let none: Retained<NSArray<NSNumber>> = NSArray::new();
                CATapDescription::initStereoGlobalTapButExcludeProcesses(
                    CATapDescription::alloc(),
                    &none,
                )
            }
            SourceSelector::Process(object_id) => {
                let processes = NSArray::from_retained_slice(&[NSNumber::new_u32(*object_id)]);
                CATapDescription::initStereoMixdownOfProcesses(
                    CATapDescription::alloc(),
                    &processes,
                )
            }
        }
    };

    // SAFETY: plain property setters on a live object.
    unsafe {
        description.setName(&NSString::from_str("Marswind Capture"));
        // Private keeps the tap out of other apps' device lists.
        description.setPrivate(true);
        // The user must keep hearing what we are transcribing.
        description.setMuteBehavior(CATapMuteBehavior::Unmuted);
    }

    Ok(description)
}

fn create_aggregate_device(tap_uid: &str) -> Result<AudioObjectID, AudioError> {
    let output_uid = default_output_device_uid()?;
    let aggregate_uid = format!("com.marswind.capture.{}", NSUUID::new().UUIDString());

    let sub_device = ns_dict(&[(kAudioSubDeviceUIDKey, ns_string(&output_uid))]);
    let tap_entry = ns_dict(&[
        (kAudioSubTapUIDKey, ns_string(tap_uid)),
        (kAudioSubTapDriftCompensationKey, ns_bool(true)),
    ]);

    let description = ns_dict(&[
        (
            kAudioAggregateDeviceNameKey,
            ns_string("Marswind Capture Device"),
        ),
        (kAudioAggregateDeviceUIDKey, ns_string(&aggregate_uid)),
        (
            kAudioAggregateDeviceMainSubDeviceKey,
            ns_string(&output_uid),
        ),
        (kAudioAggregateDeviceIsPrivateKey, ns_bool(true)),
        (kAudioAggregateDeviceIsStackedKey, ns_bool(false)),
        (kAudioAggregateDeviceTapAutoStartKey, ns_bool(true)),
        (
            kAudioAggregateDeviceSubDeviceListKey,
            as_any(NSArray::from_retained_slice(&[sub_device])),
        ),
        (
            kAudioAggregateDeviceTapListKey,
            as_any(NSArray::from_retained_slice(&[tap_entry])),
        ),
    ]);

    // NSDictionary and CFDictionary are the same object across the toll-free
    // bridge.
    let cf_description: &CFDictionary =
        unsafe { &*(Retained::as_ptr(&description) as *const CFDictionary) };

    let mut aggregate_id: AudioObjectID = 0;
    // SAFETY: the dictionary is live for the duration of the call and the out
    // pointer is valid.
    let status = unsafe {
        AudioHardwareCreateAggregateDevice(cf_description, NonNull::from(&mut aggregate_id))
    };
    check(status, "AudioHardwareCreateAggregateDevice")?;

    Ok(aggregate_id)
}

fn io_block(sink: SampleSink) -> IoBlock {
    RcBlock::new(
        move |_now: NonNull<AudioTimeStamp>,
              input: NonNull<AudioBufferList>,
              _input_time: NonNull<AudioTimeStamp>,
              _output: NonNull<AudioBufferList>,
              _output_time: NonNull<AudioTimeStamp>| {
            // SAFETY: Core Audio guarantees the buffer list is valid for the
            // duration of this callback.
            let list = unsafe { input.as_ref() };
            let count = list.mNumberBuffers as usize;
            if count == 0 {
                return;
            }

            let buffers = unsafe { std::slice::from_raw_parts(list.mBuffers.as_ptr(), count) };

            if count == 1 {
                // One buffer holding interleaved channels.
                let buffer = &buffers[0];
                if buffer.mData.is_null() {
                    return;
                }
                let samples = unsafe {
                    std::slice::from_raw_parts(
                        buffer.mData as *const f32,
                        buffer.mDataByteSize as usize / std::mem::size_of::<f32>(),
                    )
                };
                sink.push_interleaved(samples, buffer.mNumberChannels.max(1) as usize);
                return;
            }

            // One buffer per channel: mix without allocating on the audio thread.
            let mut planes: [&[f32]; MAX_PLANES] = [&[]; MAX_PLANES];
            let mut plane_count = 0;
            for buffer in buffers.iter().take(MAX_PLANES) {
                if buffer.mData.is_null() {
                    continue;
                }
                planes[plane_count] = unsafe {
                    std::slice::from_raw_parts(
                        buffer.mData as *const f32,
                        buffer.mDataByteSize as usize / std::mem::size_of::<f32>(),
                    )
                };
                plane_count += 1;
            }
            sink.push_planar(&planes[..plane_count]);
        },
    )
}

fn tap_format(tap_id: AudioObjectID) -> Result<CaptureFormat, AudioError> {
    let asbd = fixed_property::<AudioStreamBasicDescription>(tap_id, kAudioTapPropertyFormat)?;

    if asbd.mSampleRate <= 0.0 {
        return Err(AudioError::Other(
            "tap reported an invalid sample rate".into(),
        ));
    }

    Ok(CaptureFormat {
        sample_rate: asbd.mSampleRate as u32,
        channels: asbd.mChannelsPerFrame.max(1) as u16,
    })
}

fn default_output_device_uid() -> Result<String, AudioError> {
    let device =
        fixed_property::<AudioObjectID>(SYSTEM_OBJECT, kAudioHardwarePropertyDefaultOutputDevice)?;
    cfstring_property(device, kAudioDevicePropertyDeviceUID)
        .ok_or_else(|| AudioError::Other("no default output device found".into()))
}

fn process_object_ids() -> Result<Vec<AudioObjectID>, AudioError> {
    let address = property_address(kAudioHardwarePropertyProcessObjectList);

    let mut size: u32 = 0;
    // SAFETY: address and out pointer are valid, no qualifier needed.
    let status = unsafe {
        AudioObjectGetPropertyDataSize(
            SYSTEM_OBJECT,
            NonNull::from(&address),
            0,
            null(),
            NonNull::from(&mut size),
        )
    };
    check(status, "process object list size")?;

    let count = size as usize / std::mem::size_of::<AudioObjectID>();
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut ids = vec![0 as AudioObjectID; count];
    // SAFETY: the buffer is sized from the query above.
    let status = unsafe {
        AudioObjectGetPropertyData(
            SYSTEM_OBJECT,
            NonNull::from(&address),
            0,
            null(),
            NonNull::from(&mut size),
            NonNull::new(ids.as_mut_ptr() as *mut c_void).unwrap(),
        )
    };
    check(status, "process object list")?;

    Ok(ids)
}

/// Reads a fixed-size property. `T` must be a plain-old-data type matching what
/// the HAL returns for `selector` - every caller here passes a Core Audio
/// struct or integer, which are exactly that.
fn fixed_property<T>(
    object: AudioObjectID,
    selector: AudioObjectPropertySelector,
) -> Result<T, AudioError> {
    let address = property_address(selector);
    let mut size = std::mem::size_of::<T>() as u32;
    // SAFETY: T is POD, and the HAL overwrites the whole buffer on success.
    let mut value: T = unsafe { std::mem::zeroed() };

    // SAFETY: the buffer is exactly one T, which is the size the HAL expects for
    // these scalar properties.
    let status = unsafe {
        AudioObjectGetPropertyData(
            object,
            NonNull::from(&address),
            0,
            null(),
            NonNull::from(&mut size),
            NonNull::new(&mut value as *mut T as *mut c_void).unwrap(),
        )
    };
    check(status, "AudioObjectGetPropertyData")?;

    Ok(value)
}

fn cfstring_property(
    object: AudioObjectID,
    selector: AudioObjectPropertySelector,
) -> Option<String> {
    let address = property_address(selector);
    let mut size = std::mem::size_of::<*const CFString>() as u32;
    let mut value: *const CFString = null();

    // SAFETY: the out buffer holds exactly one CFStringRef.
    let status = unsafe {
        AudioObjectGetPropertyData(
            object,
            NonNull::from(&address),
            0,
            null(),
            NonNull::from(&mut size),
            NonNull::new(&mut value as *mut *const CFString as *mut c_void).unwrap(),
        )
    };
    if status != 0 {
        return None;
    }

    let ptr = NonNull::new(value as *mut CFString)?;
    // SAFETY: the HAL follows the Create rule here, so we own this reference.
    let string = unsafe { CFRetained::from_raw(ptr) };
    Some(string.to_string())
}

fn property_address(selector: AudioObjectPropertySelector) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    }
}

/// Erases the concrete class so mixed-value dictionaries can be built. Every
/// Objective-C object is an `AnyObject`, so this never fails.
fn as_any<T: Message>(object: Retained<T>) -> Retained<AnyObject> {
    unsafe { Retained::cast_unchecked(object) }
}

fn ns_string(value: &str) -> Retained<AnyObject> {
    as_any(NSString::from_str(value))
}

fn ns_bool(value: bool) -> Retained<AnyObject> {
    as_any(NSNumber::new_bool(value))
}

fn ns_dict(
    entries: &[(&CStr, Retained<AnyObject>)],
) -> Retained<NSDictionary<NSString, AnyObject>> {
    let keys: Vec<Retained<NSString>> = entries
        .iter()
        .map(|(key, _)| NSString::from_str(&key.to_string_lossy()))
        .collect();
    let key_refs: Vec<&NSString> = keys.iter().map(|k| &**k).collect();
    let value_refs: Vec<&AnyObject> = entries.iter().map(|(_, value)| &**value).collect();

    NSDictionary::from_slices(&key_refs, &value_refs)
}

/// Decides whether a process belongs in the picker.
///
/// Core Audio reports every client of the audio system, which on a normal Mac
/// is two dozen background daemons the user has no reason to transcribe. What
/// stays: anything currently playing, and any non-Apple application that could
/// start playing.
fn is_worth_listing(bundle_id: Option<&str>, active: bool) -> bool {
    if active {
        return bundle_id != Some(OWN_BUNDLE_ID);
    }

    match bundle_id {
        None => false,
        Some(OWN_BUNDLE_ID) => false,
        Some(id) => !id.starts_with("com.apple."),
    }
}

/// "com.google.Chrome" reads better as "Chrome" in a source list. Browsers play
/// audio from helper processes, so "com.google.Chrome.helper" has to resolve to
/// "Chrome" too rather than a list full of identical "helper" entries.
fn display_name_for_bundle(bundle_id: &str) -> String {
    const GENERIC: [&str; 4] = ["helper", "service", "agent", "app"];

    let mut parts: Vec<&str> = bundle_id.split('.').filter(|p| !p.is_empty()).collect();
    while parts.len() > 1 && GENERIC.contains(&parts[parts.len() - 1].to_lowercase().as_str()) {
        parts.pop();
    }

    parts
        .last()
        .map(|part| part.replace(['-', '_'], " "))
        .unwrap_or_else(|| bundle_id.to_string())
}

fn check(status: i32, op: &str) -> Result<(), AudioError> {
    if status == 0 {
        return Ok(());
    }

    let fourcc = fourcc(status);
    // The HAL reports a denied tap as an illegal operation rather than a
    // dedicated permission error.
    if fourcc == "nope" || fourcc == "!pri" || fourcc == "!obj" && op.contains("Tap") {
        return Err(AudioError::PermissionDenied);
    }

    Err(AudioError::Os {
        op: op.to_string(),
        code: status,
        fourcc,
    })
}

/// Core Audio statuses are usually four packed ASCII characters.
fn fourcc(status: i32) -> String {
    let bytes = (status as u32).to_be_bytes();
    if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        status.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortens_bundle_identifiers() {
        assert_eq!(display_name_for_bundle("com.google.Chrome"), "Chrome");
        assert_eq!(display_name_for_bundle("com.apple.Music"), "Music");
        assert_eq!(display_name_for_bundle("Firefox"), "Firefox");
    }

    #[test]
    fn resolves_helper_processes_to_their_application() {
        assert_eq!(
            display_name_for_bundle("com.google.Chrome.helper"),
            "Chrome"
        );
        assert_eq!(
            display_name_for_bundle("com.microsoft.teams2.helper"),
            "teams2"
        );
        assert_eq!(display_name_for_bundle("us.zoom.xos"), "xos");
    }

    #[test]
    fn hides_idle_system_daemons_but_keeps_playing_ones() {
        assert!(!is_worth_listing(Some("com.apple.audiomxd"), false));
        assert!(is_worth_listing(Some("com.apple.Music"), true));
        assert!(is_worth_listing(Some("com.google.Chrome.helper"), false));
        assert!(!is_worth_listing(None, false));
        assert!(is_worth_listing(None, true));
    }

    #[test]
    fn never_lists_marswind_itself() {
        assert!(!is_worth_listing(Some(OWN_BUNDLE_ID), false));
        assert!(!is_worth_listing(Some(OWN_BUNDLE_ID), true));
    }

    #[test]
    fn decodes_four_character_status_codes() {
        // 'nope' = kAudioHardwareIllegalOperationError
        let status = i32::from_be_bytes(*b"nope");
        assert_eq!(fourcc(status), "nope");
    }

    #[test]
    fn maps_illegal_operation_to_permission_denied() {
        let status = i32::from_be_bytes(*b"nope");
        let err = check(status, "AudioHardwareCreateProcessTap").unwrap_err();
        assert!(matches!(err, AudioError::PermissionDenied));
    }

    #[test]
    fn zero_status_is_success() {
        assert!(check(0, "anything").is_ok());
    }

    #[test]
    fn lists_at_least_the_system_source() {
        let sources = list_sources().expect("listing sources should not fail");
        assert!(sources.iter().any(|s| s.id == "system"));
        assert_eq!(sources[0].kind, SourceKind::System);
    }
}
