"use client";
import { Link } from "@/components/catalyst-ui/link";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Circle, Speaker } from "lucide-react";
import { useRouter } from "next/router";
import { Slider } from "@/components/ui/slider";
import AudioRecorder from "@/components/audio-recorder/AudioRecorder";
import { Separator } from "@/components/ui/separator";
import {
  useCompleteTranscription,
  useLiveTranscription,
} from "@/hooks/useTranscription";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import clsx from "clsx";

const transcript = [
  {
    speaker: "Speaker 1",
    text: "Houston, we've had a problem here.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 2",
    text: "This is Houston. Say again please.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 1",
    text: "Uh Houston we've had a problem. We've had a main beam plus one volt.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 2",
    text: "Roger main beam interval.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 1",
    text: "Uh uh uh",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 2",
    text: "Speaker 2 So okay stand, by thirteen we're looking at it.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 1",
    text: " Okay uh right now uh Houston the uh voltage is uh is looking good um And we had a apretty large bank or so.",
    timeStamp: 0,
  },
];

export default function Page() {
  const [isRecording, setIsRecording] = useState(false);

  const liveTranscription = useLiveTranscription(isRecording);
  const completeTranscription = useCompleteTranscription(isRecording);
  const startRecording = async () => {
    setIsRecording(true);
    await invoke("start_recording", {
      options: { user_id: "1", audio_name: "name" },
    }).catch(() => setIsRecording(false));
  };

  const stopRecording = () => {
    setIsRecording(false);
    invoke("stop_recording");
  };

  const transcription = isRecording ? liveTranscription : completeTranscription;

  return (
    <div className="p-2 h-screen">
      <Card>
        <CardHeader>
          <CardTitle>Your Converstation</CardTitle>
        </CardHeader>
        <CardContent className="flex-1 overflow-y-scroll">
          <Separator />
          {transcription.data?.full_text?.map((slice, i) => (
            <div className="mt-2 flex flex-col" key={slice}>
              <p>{slice}</p>
            </div>
          ))}
        </CardContent>
        <CardFooter className="flex flex-col gap-4">
          <Separator />

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
        </CardFooter>
      </Card>
    </div>
  );
}
