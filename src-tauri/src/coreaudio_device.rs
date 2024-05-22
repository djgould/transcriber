extern crate core_foundation as cf;
extern crate coreaudio;
extern crate coreaudio_sys;
extern crate libc;

use cf::base::{CFAllocatorRef, CFRelease, CFTypeRef};
use cf::string::{kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringRef};
use coreaudio::sys::*;
use libc::c_void;
use std::ffi::CString;
use std::ptr;

fn get_audio_device_id_from_uid(device_uid: &str) -> AudioDeviceID {
    let mut device_id: coreaudio_sys::AudioDeviceID = coreaudio_sys::kAudioObjectUnknown;
    let mut size = std::mem::size_of::<coreaudio_sys::AudioDeviceID>() as u32;
    let property_address = coreaudio_sys::AudioObjectPropertyAddress {
        mSelector: coreaudio_sys::kAudioHardwarePropertyTranslateUIDToDevice,
        mScope: coreaudio_sys::kAudioObjectPropertyScopeGlobal,
        mElement: coreaudio_sys::kAudioObjectPropertyElementMaster,
    };

    let uid = CString::new(device_uid).unwrap();
    let cf_uid = unsafe {
        CFStringCreateWithCString(
            cf::base::kCFAllocatorDefault,
            uid.as_ptr(),
            kCFStringEncodingUTF8,
        )
    };

    unsafe {
        let status = coreaudio_sys::AudioObjectGetPropertyData(
            coreaudio_sys::kAudioObjectSystemObject,
            &property_address,
            std::mem::size_of::<CFStringRef>() as u32,
            &cf_uid as *const _ as *const c_void,
            &mut size,
            &mut device_id as *mut _ as *mut c_void,
        );

        if coreaudio::Error::from_os_status(status).is_err() {
            eprintln!("Error translating UID to device ID: {}", status);
        }

        CFRelease(cf_uid as CFTypeRef);
    }

    device_id
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <device_uid>", args[0]);
        std::process::exit(1);
    }

    let device_uid = &args[1];
    println!("Attempting to get AudioDeviceID for UID: {}", device_uid);

    // list_all_audio_devices();

    let device_id = get_audio_device_id_from_uid(device_uid);
    if device_id == coreaudio_sys::kAudioObjectUnknown {
        eprintln!("Invalid device UID: {}", device_uid);
        std::process::exit(1);
    }

    println!("Successfully got AudioDeviceID: {}", device_id);
}
