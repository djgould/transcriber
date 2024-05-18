import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export function useAudioDevicesQuery() {
  return useQuery({
    queryKey: ["audio_devices"],
    queryFn: async (): Promise<string[]> => {
      return await invoke("enumerate_audio_devices");
    },
  });
}
