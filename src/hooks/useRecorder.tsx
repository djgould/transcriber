import {
  selectedAudioInputDeviceAtom,
  selectedAudioOutputDeviceAtom,
} from "@/atoms/audioDeviceAtom";
import { useToast } from "@/components/ui/use-toast";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useAtom } from "jotai";

export function useRecorderMutation() {
  const [selectedAudioDevice] = useAtom(selectedAudioInputDeviceAtom);
  const recordingMutation = useMutation({
    mutationFn: async ({ command }: { command: "start" | "stop" }) => {
      switch (command) {
        case "start":
          return await invoke("start_recording", {
            options: { user_id: "1", audio_name: selectedAudioDevice },
          });
        case "stop":
          return await invoke("stop_recording");
      }
    },
  });

  return recordingMutation;
}

export function useStartRecorderMutation() {
  const queryClient = useQueryClient();
  const [selectedAudioInputDevice] = useAtom(selectedAudioInputDeviceAtom);
  const [selectedAudioOutputDevice] = useAtom(selectedAudioOutputDeviceAtom);
  const { toast } = useToast();
  const recordingMutation = useMutation({
    mutationFn: async ({ conversation_id }: { conversation_id: number }) => {
      if (!selectedAudioInputDevice) {
        throw new Error("No audio input device selected");
      }
      if (!selectedAudioOutputDevice) {
        throw new Error("No audio output device selected");
      }
      return await invoke("start_recording", {
        options: {
          user_id: "1",
          audio_input_name: selectedAudioInputDevice,
          audio_output_name: selectedAudioOutputDevice,
        },
        conversationId: conversation_id,
      });
    },
    onSuccess() {
      queryClient.invalidateQueries({
        queryKey: ["is_recording"],
      });
    },
    onError: (error) => {
      toast({
        title: "Error",
        description: error.message,
      });
    },
  });

  return recordingMutation;
}

export function useStopRecorderMutation() {
  const queryClient = useQueryClient();
  const recordingMutation = useMutation({
    mutationFn: async () => {
      return await invoke("stop_recording");
    },
    onSuccess() {
      queryClient.invalidateQueries({
        queryKey: ["is_recording"],
      });
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

export function useIsRecording() {
  const isRecordingQuery = useQuery({
    queryKey: ["is_recording"],
    queryFn: async () => {
      return await invoke<boolean>("is_recording");
    },
  });

  return isRecordingQuery;
}
