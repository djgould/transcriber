import { selectedAudioDeviceAtom } from "@/atoms/audioDeviceAtom";
import { useToast } from "@/components/ui/use-toast";
import { useAudioDevice } from "@/context/AudioDeviceContext";
import { useMutation, useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useAtom } from "jotai";

export function useRecorderMutation() {
  const [selectedAudioDevice] = useAtom(selectedAudioDeviceAtom);
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
  const [selectedAudioDevice] = useAtom(selectedAudioDeviceAtom);
  const { toast } = useToast();
  const recordingMutation = useMutation({
    mutationFn: async ({ conversation_id }: { conversation_id: number }) => {
      if (!selectedAudioDevice) {
        throw new Error("No audio device selected");
      }
      console.log(conversation_id);
      return await invoke("start_recording", {
        options: {
          user_id: "1",
          audio_name: selectedAudioDevice,
        },
        conversationId: conversation_id,
      });
    },
    onError: (error) => {
      toast({
        title: "Error",
        description: error,
      });
    },
  });

  return recordingMutation;
}

export function useStopRecorderMutation() {
  const recordingMutation = useMutation({
    mutationFn: async ({ conversation_id }: { conversation_id: number }) => {
      return await invoke("stop_recording", {
        conversationId: conversation_id,
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
