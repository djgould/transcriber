use coreaudio::audio_unit::{
    macos_helpers::{
        get_audio_device_ids_for_scope, get_audio_device_supports_scope, get_device_id_from_name,
        get_device_name,
    },
    Scope,
};
use cpal::traits::{DeviceTrait, HostTrait};
use log::info;
use uuid::Uuid;

use crate::{
    audio::macos::{aggregate_device::create_output_aggregate_device, helpers::get_device_uid},
    DeviceState,
};
use std::sync::Arc;

#[tauri::command]
pub async fn set_output_device_name(
    state: tauri::State<'_, Arc<tauri::async_runtime::Mutex<DeviceState>>>,
    name: String,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    let name_clone = name.clone();
    guard.selected_output_name = Some(name);
    let device_id =
        get_device_id_from_name(&name_clone, false).expect("failed to get device id from name");
    let device_uid = get_device_uid(device_id).expect("failed to get device uid");
    let result =
        create_output_aggregate_device(&device_uid, "Platy Speaker", &Uuid::new_v4().to_string())
            .expect("Failed to create aggregate device");
    info!(
        "updated output device {} aggregate id: {} tap id: {}",
        guard.selected_output_name.as_ref().unwrap(),
        result.aggregate_device_id,
        result.tap_id,
    );
    guard.aggregate_device_id = Some(result.aggregate_device_id);
    guard.tap_id = Some(result.tap_id);
    Ok(())
}

#[tauri::command]
pub async fn set_input_device_name(
    state: tauri::State<'_, Arc<tauri::async_runtime::Mutex<DeviceState>>>,
    name: String,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    println!("Setting input device name: {}", name);
    let name_clone = name.clone();
    guard.selected_input_name = Some(name);
    guard
        .active_listener
        .unregister()
        .expect("Failed to unregister device listener");

    let device_id =
        get_device_id_from_name(&name_clone, true).expect("Failed to get device from name");

    guard
        .active_listener
        .register(device_id)
        .expect("Failed to register listener");

    Ok(())
}

#[tauri::command]
pub fn enumerate_audio_output_devices() -> Vec<String> {
    let all_devices =
        get_audio_device_ids_for_scope(Scope::Output).expect("failed to get device ids");
    let output_devices: Vec<String> = all_devices
        .into_iter()
        .filter(|device| {
            get_audio_device_supports_scope(*device, Scope::Output)
                .expect("failed to see if device supports scope")
        })
        .map(|device| get_device_name(device).ok().unwrap())
        .filter(|device_name| device_name != "Platy Speaker")
        .collect();

    output_devices
}

#[tauri::command]
pub fn enumerate_audio_input_devices() -> Vec<String> {
    for host in cpal::ALL_HOSTS {
        println!("host {}", host.name());
    }

    let host = cpal::default_host();
    let default_device = host
        .default_input_device()
        .expect("No default input device available");
    let default_device_name = default_device
        .name()
        .expect("Failed to get default device name");

    let devices = host.input_devices().expect("Failed to get devices");
    println!("Logging devices");
    let mut input_device_names: Vec<String> = devices
        .filter_map(|device| {
            println!("{}", device.name().unwrap());
            let supported_input_configs = device.supported_input_configs();
            if supported_input_configs.is_ok() && supported_input_configs.unwrap().count() > 0 {
                device.name().ok()
            } else {
                None
            }
        })
        .collect();

    input_device_names.retain(|name| {
        name != &default_device_name && name != "Platy Speaker" && name != "Platy Microphone"
    });
    input_device_names.insert(0, default_device_name);

    input_device_names
}
