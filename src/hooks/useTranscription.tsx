import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export function useLiveTranscription(isRecording: boolean) {
  return useQuery({
    queryKey: ["liveTranscription"],
    queryFn: async () => {
      invoke("get_live_transcription");
    },
    enabled: isRecording,
  });
}
