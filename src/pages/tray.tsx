"use client";
import {
  useAudioInputDevicesQuery,
  useAudioOutputDevicesQuery,
} from "@/hooks/useMediaDevices";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useAtom } from "jotai";
import {
  selectedAudioInputDeviceAtom,
  selectedAudioOutputDeviceAtom,
} from "@/atoms/audioDeviceAtom";
import { ReactElement, useEffect, useState } from "react";
import {
  useIsRecording,
  useStartRecorderMutation,
  useStopRecorderMutation,
} from "@/hooks/useRecorder";
import { Circle, Eye, Loader, Trash } from "lucide-react";
import { Button } from "@/components/ui/button";
import clsx from "clsx";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import {
  useConversations,
  useCreateConversationMutation,
  useDeleteConversationMutation,
} from "@/hooks/useConversations";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import Link from "next/link";
import { invoke } from "@tauri-apps/api/core";
import NavBar from "@/components/nav/NavBar";
import { useRouter } from "next/router";
import useIsTray from "@/hooks/useIsTray";
import { NextPageWithLayout } from "./_app";
import { TrayLayout } from "@/components/layout/tray";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const Page: NextPageWithLayout = () => {
  const isTray = useIsTray();
  const backPath = isTray ? "/tray" : "/main";

  const audioInputDevices = useAudioInputDevicesQuery();
  const audioOutputDevices = useAudioOutputDevicesQuery();

  const [selectedAudioInputDevice, setSelectedAudioInputDevice] = useAtom(
    selectedAudioInputDeviceAtom
  );
  const [selectedAudioOutputDevice, setSelectedAudioOutputDevice] = useAtom(
    selectedAudioOutputDeviceAtom
  );
  const [activeRecordingInfo, setActiveRecordingInfo] = useState<
    { conversation_id: number; status: "recording" | "stopping" } | undefined
  >();

  const startRecorderMutation = useStartRecorderMutation();
  const stopRecorderMutation = useStopRecorderMutation();
  const isRecording = useIsRecording();

  const conversations = useConversations();
  const createConversationMutation = useCreateConversationMutation();
  const deleteConversationMutation = useDeleteConversationMutation();

  const viewConversation = async (conversationId: number) => {
    await invoke("open_conversation", { conversationId });
  };

  const startRecording = async () => {
    createConversationMutation.mutate(undefined, {
      onSuccess(conversation) {
        setActiveRecordingInfo({
          conversation_id: conversation.id,
          status: "recording",
        });
        startRecorderMutation.mutate(
          {
            conversation_id: conversation.id,
          },
          {
            onError: () => {
              setActiveRecordingInfo(undefined);
            },
          }
        );
      },
    });
  };

  const stopRecording = () => {
    if (!activeRecordingInfo?.conversation_id) return;
    setActiveRecordingInfo({
      ...activeRecordingInfo,
      status: "stopping",
    });
    stopRecorderMutation.mutate(
      { conversation_id: activeRecordingInfo?.conversation_id },
      {
        onSuccess: () => {
          setActiveRecordingInfo(undefined);
        },
      }
    );
  };

  return (
    <div className="p-2 h-screen flex flex-col gap-4">
      <Select
        value={selectedAudioInputDevice}
        onValueChange={(value) => {
          invoke("set_input_device_name", { name: value });
          setSelectedAudioInputDevice(value);
        }}
      >
        <SelectTrigger className="w-full">
          <SelectValue placeholder={selectedAudioInputDevice} />
        </SelectTrigger>
        <SelectContent>
          {audioInputDevices.data?.map((device) => (
            <SelectItem value={device} key={device}>
              {device}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Select
        value={selectedAudioOutputDevice}
        onValueChange={(value) => {
          invoke("set_output_device_name", { name: value });
          setSelectedAudioOutputDevice(value);
        }}
      >
        <SelectTrigger className="w-full">
          <SelectValue placeholder={selectedAudioOutputDevice} />
        </SelectTrigger>
        <SelectContent>
          {audioOutputDevices.data?.map((device) => (
            <SelectItem value={device} key={device}>
              {device}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Button
        variant="outline"
        disabled={activeRecordingInfo?.status === "stopping"}
        onClick={() => {
          if (activeRecordingInfo) {
            stopRecording();
          } else {
            startRecording();
          }
        }}
      >
        {!startRecorderMutation.isPending &&
        !stopRecorderMutation.isPending &&
        isRecording.data ? (
          <Circle className={clsx("text-red-800 fill-red-800 animate-pulse")} />
        ) : (
          startRecorderMutation.isPending ||
          (stopRecorderMutation.isPending ? (
            <Loader className="animate-spin" />
          ) : (
            <Circle className={clsx("text-red-800")} />
          ))
        )}
      </Button>
      <Card>
        <CardHeader>
          <CardTitle className="text-lg text-center">
            Your Converstations
          </CardTitle>
        </CardHeader>
        <CardContent className="flex-1 overflow-y-scroll">
          <Table>
            <TableCaption>A list of your recent conversations.</TableCaption>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[100px]">Created at</TableHead>
                <TableHead className="w-[100px]">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {conversations.data?.map((conversation: any) => (
                <TableRow key={conversation.id}>
                  <TableCell className="font-medium">
                    {new Date(conversation.created_at).toLocaleDateString()}
                  </TableCell>
                  <TableCell className="font-medium flex justify-between">
                    <Button
                      size={"sm"}
                      variant={"secondary"}
                      onClick={() => viewConversation(conversation.id)}
                    >
                      <Eye />
                    </Button>

                    <Button
                      size={"sm"}
                      variant={"secondary"}
                      onClick={() => {
                        deleteConversationMutation.mutate({
                          conversationId: conversation.id,
                        });
                      }}
                    >
                      <Trash />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
        <CardFooter className="flex flex-col gap-4"></CardFooter>
      </Card>
    </div>
  );
};

Page.getLayout = function getLayout(page: ReactElement) {
  return <TrayLayout>{page}</TrayLayout>;
};

export default Page;
