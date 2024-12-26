import { useCreateConversationMutation } from "@/hooks/useConversations";
import {
  useIsRecording,
  useStartRecorderMutation,
  useStopRecorderMutation,
} from "@/hooks/useRecorder";
import { Button } from "../ui/button";
import { Circle, Loader, Mic } from "lucide-react";
import clsx from "clsx";

interface RecordingButtonProps {
  variant?: "main" | "tray";
}

export function RecordingButton({ variant = "tray" }: RecordingButtonProps) {
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

  if (variant === "tray") {
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

  return (
    <Button
      variant={isRecording.data ? "destructive" : "ghost"}
      size="icon"
      disabled={stopRecorderMutation.isPending}
      onClick={() => {
        if (isRecording.data) {
          stopRecording();
        } else {
          startRecording();
        }
      }}
      className={`transition-colors duration-200 ${
        isRecording.data
          ? "bg-red-600 hover:bg-red-700 text-white"
          : "text-white hover:bg-gray-800"
      }`}
    >
      <Mic className={`h-5 w-5 ${isRecording.data ? "animate-pulse" : ""}`} />
      <span className="sr-only">
        {isRecording.data ? "Stop recording" : "Start recording"}
      </span>
    </Button>
  );
}
