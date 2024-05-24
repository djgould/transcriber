use std::{
    mem,
    os::raw::c_void,
    ptr::null,
    sync::atomic::{AtomicBool, Ordering},
};

use coreaudio::{audio_unit::macos_helpers::get_default_device_id, Error};
use coreaudio_sys::{
    kAudioDevicePropertyDeviceIsRunningSomewhere, kAudioObjectPropertyElementMaster,
    kAudioObjectPropertyScopeGlobal, AudioDeviceID, AudioObjectAddPropertyListener,
    AudioObjectGetPropertyData, AudioObjectID, AudioObjectPropertyAddress,
    AudioObjectRemovePropertyListener, OSStatus,
};
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

/// An ActiveListener is used to get notified when a device is disconnected.
pub struct ActiveListener {
    alive: Box<AtomicBool>,
    device_id: AudioDeviceID,
    property_address: AudioObjectPropertyAddress,
    alive_listener: Option<
        unsafe extern "C" fn(u32, u32, *const AudioObjectPropertyAddress, *mut c_void) -> i32,
    >,
    sender: watch::Sender<bool>,
}

impl Drop for ActiveListener {
    fn drop(&mut self) {
        let _ = self.unregister();
    }
}

impl ActiveListener {
    /// Create a new ActiveListener for the given AudioDeviceID.
    /// The listener must be registered by calling `register()` in order to start receiving notifications.
    pub fn new(device_id: AudioDeviceID, sender: watch::Sender<bool>) -> ActiveListener {
        // Add our listener callback.
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceIsRunningSomewhere,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster,
        };
        ActiveListener {
            alive: Box::new(AtomicBool::new(false)),
            device_id,
            property_address,
            alive_listener: None,
            sender,
        }
    }

    /// Register this listener to receive notifications.
    pub fn register(&mut self) -> Result<(), Error> {
        unsafe extern "C" fn alive_listener(
            device_id: AudioObjectID,
            _n_addresses: u32,
            _properties: *const AudioObjectPropertyAddress,
            self_ptr: *mut ::std::os::raw::c_void,
        ) -> OSStatus {
            let self_ptr: &mut ActiveListener = &mut *(self_ptr as *mut ActiveListener);
            let alive: u32 = 0;
            let data_size = mem::size_of::<u32>();
            let property_address = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyDeviceIsRunningSomewhere,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMaster,
            };
            let result = AudioObjectGetPropertyData(
                device_id,
                &property_address as *const _,
                0,
                null(),
                &data_size as *const _ as *mut _,
                &alive as *const _ as *mut _,
            );
            self_ptr.alive.store(alive > 0, Ordering::SeqCst);
            self_ptr.sender.send(alive > 0).ok();
            result
        }

        // Add our listener callback.
        let status = unsafe {
            AudioObjectAddPropertyListener(
                self.device_id,
                &self.property_address as *const _,
                Some(alive_listener),
                self as *const _ as *mut _,
            )
        };
        Error::from_os_status(status)?;
        self.alive_listener = Some(alive_listener);
        Ok(())
    }

    /// Unregister this listener to stop receiving notifications
    pub fn unregister(&mut self) -> Result<(), Error> {
        if self.alive_listener.is_some() {
            let status = unsafe {
                AudioObjectRemovePropertyListener(
                    self.device_id,
                    &self.property_address as *const _,
                    self.alive_listener,
                    self as *const _ as *mut _,
                )
            };
            Error::from_os_status(status)?;
            self.alive_listener = None;
        }
        Ok(())
    }

    /// Check if the device is still alive.
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}
