import { useCreateConversationMutation } from "@/hooks/useConversations";
import {
  useIsRecording,
  useStartRecorderMutation,
  useStopRecorderMutation,
} from "@/hooks/useRecorder";
import { Button } from "../ui/button";
import { Circle, Loader } from "lucide-react";
import clsx from "clsx";

export function RecordingButton() {
  const createConversationMutation = useCreateConversationMutation();
  const startRecorderMutation = useStartRecorderMutation();
  const stopRecorderMutation = useStopRecorderMutation();
  const isRecording = useIsRecording();

  const startRecording = async () => {
    createConversationMutation.mutate(undefined, {
      onSuccess(conversation) {
        startRecorderMutation.mutate({
          conversation_id: conversation.id,
        });
      },
    });
  };

  const stopRecording = () => {
    stopRecorderMutation.mutate();
  };

  if (startRecorderMutation.isPending || stopRecorderMutation.isPending) {
    <Button
      variant="outline"
      disabled={stopRecorderMutation.isPending}
      onClick={() => {
        if (isRecording.data) {
          stopRecording();
        } else {
          startRecording();
        }
      }}
    >
      <Loader className="animate-spin" />
    </Button>;
  }

  return (
    <Button
      variant="outline"
      disabled={stopRecorderMutation.isPending}
      onClick={() => {
        if (isRecording.data) {
          stopRecording();
        } else {
          startRecording();
        }
      }}
    >
      {isRecording.data ? (
        <Circle className={clsx("text-red-800 fill-red-800 animate-pulse")} />
      ) : (
        <Circle className={clsx("text-red-800")} />
      )}
    </Button>
  );
}
