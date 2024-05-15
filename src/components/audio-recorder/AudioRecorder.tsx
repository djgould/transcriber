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

function AudioRecorder() {
  const [isRecording, setIsRecording] = useState(false);
  const [chunks, setChunks] = useState<Blob[]>([]);
  const [clips, setClips] = useState<
    { clipName: string | null; audioURL: string }[]
  >([]);
  const [mediaPort, setMediaPort] = useState<MessagePort>();
  const mediaRecorderRef = useRef<IMediaRecorder | null>(null);

  // async function setup() {
  //   try {
  //     const _mediaPort = await connect();
  //     setMediaPort(_mediaPort);
  //     await register(_mediaPort);
  //   } catch (err) {
  //     console.error(err);
  //   }
  //   navigator.mediaDevices.enumerateDevices().then(console.log);
  //   if (navigator.mediaDevices) {
  //     console.log(chunks.length);
  //     console.log("getUserMedia supported.");
  //     const constraints = { audio: true };
  //     navigator.mediaDevices
  //       .getUserMedia(constraints)
  //       .then((stream) => {
  //         const mediaRecorder =
  //           mediaRecorderRef.current ||
  //           new MediaRecorder(stream, { mimeType: "audio/wav" });
  //         mediaRecorderRef.current = mediaRecorder;

  //         mediaRecorder.ondataavailable = (e) => {
  //           console.log(e.data);
  //           setChunks((prevChunks) => [...prevChunks, e.data]);
  //         };

  //         mediaRecorder.onstop = () => {
  //           const clipName = prompt("Enter a name for your sound clip");
  //           const blob = new Blob(chunks, { type: "audio/wav" });
  //           const audioURL = URL.createObjectURL(blob);
  //           console.log(audioURL);
  //           setClips((prevClips) => [...prevClips, { clipName, audioURL }]);
  //         };
  //       })
  //       .catch((err) => {
  //         console.error(`The following error occurred: ${err}`);
  //       });
  //   }
  // }

  // useEffect(() => {
  //   function disconnectMedia() {
  //     mediaPort && disconnect(mediaPort);
  //   }

  //   return disconnectMedia;
  // }, [mediaPort]);

  const startRecording = async () => {
    setIsRecording(true);
    await invoke("start_recording").catch(() => setIsRecording(false));
    // setup();
    // if (mediaRecorderRef.current) {
    //   setChunks([]);
    //   mediaRecorderRef.current.start(1000);
    //   setIsRecording(true);
    //   console.log("recorder started");
    // }
  };

  const stopRecording = () => {
    setIsRecording(false);
    invoke("stop_recording");

    // if (mediaRecorderRef.current) {
    //   mediaRecorderRef.current.stop();
    //   setIsRecording(false);
    //   console.log("recorder stopped");
    // }
  };

  // const deleteClip = (audioURL: string) => {
  //   setClips(clips.filter((clip) => clip.audioURL !== audioURL));
  // };

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
