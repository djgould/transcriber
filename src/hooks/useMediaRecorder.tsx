import React, { useState, useEffect, useRef } from "react";
import { LiveAudioVisualizer } from "react-audio-visualize";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import {
  IMediaRecorder,
  MediaRecorder,
  register,
} from "extendable-media-recorder";
import { connect } from "extendable-media-recorder-wav-encoder";

function blobToUint8Array(blob: Blob) {
  return new Promise<Uint8Array>((resolve, reject) => {
    const reader = new FileReader();

    reader.onloadend = () => {
      const arrayBuffer = reader.result;
      if (!arrayBuffer || typeof arrayBuffer === "string") return reject();
      const uint8Array = new Uint8Array(arrayBuffer);
      resolve(uint8Array);
    };

    reader.onerror = (error) => {
      reject(error);
    };

    reader.readAsArrayBuffer(blob);
  });
}

async function connectWavEncoder() {
  await register(await connect());
}

connectWavEncoder();

function AudioRecorder() {
  const [isRecording, setIsRecording] = useState(false);
  const [chunks, setChunks] = useState<Blob[]>([]);
  const [clips, setClips] = useState<
    { clipName: string | null; audioURL: string }[]
  >([]);
  const mediaRecorderRef = useRef<IMediaRecorder | null>(null);

  useEffect(() => {
    async function setup() {
      console.log("navigator");
      navigator.mediaDevices.enumerateDevices().then(console.log);
      if (navigator.mediaDevices) {
        console.log(chunks.length);
        console.log("getUserMedia supported.");
        const constraints = { audio: true };
        navigator.mediaDevices
          .getUserMedia(constraints)
          .then((stream) => {
            const mediaRecorder =
              mediaRecorderRef.current ||
              new MediaRecorder(stream, { mimeType: "audio/wav" });
            mediaRecorderRef.current = mediaRecorder;

            mediaRecorder.ondataavailable = (e) => {
              console.log(e.data);
              setChunks((prevChunks) => [...prevChunks, e.data]);
            };

            mediaRecorder.onstop = () => {
              const clipName = prompt("Enter a name for your sound clip");
              const blob = new Blob(chunks, { type: "audio/wav" });
              const audioURL = URL.createObjectURL(blob);
              console.log(audioURL);
              setClips((prevClips) => [...prevClips, { clipName, audioURL }]);
            };
          })
          .catch((err) => {
            console.error(`The following error occurred: ${err}`);
          });
      }
    }

    setup();
  }, [chunks]);

  const startRecording = () => {
    if (mediaRecorderRef.current) {
      setChunks([]);
      mediaRecorderRef.current.start(1000);
      setIsRecording(true);
      console.log("recorder started");
    }
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current) {
      mediaRecorderRef.current.stop();
      setIsRecording(false);
      console.log("recorder stopped");
    }
  };

  const deleteClip = (audioURL: string) => {
    setClips(clips.filter((clip) => clip.audioURL !== audioURL));
  };

  return (
    <div>
      <button onClick={startRecording}>Record</button>
      <button onClick={stopRecording}>Stop</button>
      {mediaRecorderRef.current && (
        <LiveAudioVisualizer
          mediaRecorder={mediaRecorderRef.current}
          width={200}
          height={75}
        />
      )}
      <div>
        {clips.map((clip, index) => (
          <div key={index} className="clip">
            <audio controls src={clip.audioURL}></audio>
            <p>{clip.clipName}</p>
            <button onClick={() => deleteClip(clip.audioURL)}>Delete</button>

            <button
              onClick={async () => {
                const filePath = await save();
                if (!filePath) return "";
                const uint8array = await blobToUint8Array(new Blob(chunks));
                await writeFile(filePath, uint8array);
              }}
            >
              Download
            </button>
          </div>
        ))}
      </div>
      {mediaRecorderRef.current?.state}
    </div>
  );
}

export default AudioRecorder;
