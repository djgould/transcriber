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
import { ReactElement, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import clsx from "clsx";
import {
  useStartRecorderMutation,
  useStopRecorderMutation,
} from "@/hooks/useRecorder";
import { MainLayout } from "@/components/layout/main";
import { NextPageWithLayout } from "@/pages/_app";

const Page: NextPageWithLayout = () => {
  const [isRecording, setIsRecording] = useState(false);

  const liveTranscription = useLiveTranscription(isRecording);
  const completeTranscription = useCompleteTranscription(isRecording);
  const startRecorderMutation = useStartRecorderMutation();
  const stopRecorderMutation = useStopRecorderMutation();

  useEffect(() => {
    invoke("enumerate_audio_devices").then(console.log);
  }, []);

  const startRecording = async () => {
    setIsRecording(true);

    startRecorderMutation.mutate();
  };

  const stopRecording = () => {
    stopRecorderMutation.mutate(null, {
      onSuccess: () => {
        setIsRecording(false);
      },
    });
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
          {stopRecorderMutation.isPending && (
            <div className="flex flex-col items-center justify-center">
              <p>Processing transcription</p>
            </div>
          )}
          {!stopRecorderMutation.isPending &&
            transcription.data?.full_text?.map((slice, i) => (
              <div className="mt-2 flex flex-col" key={`${slice}-${i}`}>
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
};

Page.getLayout = function getLayout(page: ReactElement) {
  return <MainLayout>{page}</MainLayout>;
};

export default Page;
