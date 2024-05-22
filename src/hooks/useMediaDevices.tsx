import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export function useAudioInputDevicesQuery() {
  return useQuery({
    queryKey: ["audio_input_devices"],
    queryFn: async (): Promise<string[]> => {
      return await invoke("enumerate_audio_input_devices");
    },
  });
}

export function useAudioOutputDevicesQuery() {
  return useQuery({
    queryKey: ["audio_output_devices"],
    queryFn: async (): Promise<string[]> => {
      return await invoke("enumerate_audio_output_devices");
    },
  });
}
