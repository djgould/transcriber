import React, { useState, useEffect, useRef } from "react";
import { LiveAudioVisualizer } from "react-audio-visualize";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import {
  IMediaRecorder,
  MediaRecorder,
  register,
} from "extendable-media-recorder";
import { connect, disconnect } from "extendable-media-recorder-wav-encoder";
import { Button } from "../ui/button";
import { Circle } from "lucide-react";
import clsx from "clsx";
import { invoke } from "@tauri-apps/api/core";
import { useLiveTranscription } from "@/hooks/useTranscription";

function AudioRecorder() {
  const [isRecording, setIsRecording] = useState(false);
  const liveTranscription = useLiveTranscription(isRecording);

  const startRecording = async () => {
    setIsRecording(true);
    await invoke("start_recording").catch(() => setIsRecording(false));
  };

  const stopRecording = () => {
    setIsRecording(false);
    invoke("stop_recording");
  };

  return (
    <div>
      <Button
        variant="outline"
        onClick={() => {
          if (isRecording) {
            stopRecording();
          } else {
            startRecording();
          }
        }}
      >
        <Circle
          className={clsx(
            "text-red-800",
            isRecording && " fill-red-800 animate-pulse"
          )}
        />
      </Button>
    </div>
  );
}

export default AudioRecorder;
