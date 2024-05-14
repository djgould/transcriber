import { useMutation, useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export function useRecordingMutation() {
  const recordingMutation = useMutation({
    mutationFn: async ({ command }: { command: "start" | "stop" }) => {
      switch (command) {
        case "start":
          return await invoke("start_recording");
        case "stop":
          return await invoke("stop_recording");
      }
    },
  });

  return recordingMutation;
}

export function useRecordingState() {
  const recordingState = useQuery({
    queryKey: ["recording_state"],
    queryFn: async () => {
      return await invoke("get_recording_state");
    },
  });

  return recordingState;
}
