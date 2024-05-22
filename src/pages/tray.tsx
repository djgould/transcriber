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
import { useEffect, useState } from "react";
import {
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

export default function Page() {
  const { pathname, asPath } = useRouter();
  const isTray = asPath.includes("tray");
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

  const conversations = useConversations();
  const createConversationMutation = useCreateConversationMutation();
  const deleteConversationMutation = useDeleteConversationMutation();

  const startRecording = async () => {
    createConversationMutation.mutate(undefined, {
      onSuccess(conversation) {
        setActiveRecordingInfo({
          conversation_id: conversation.lastInsertId,
          status: "recording",
        });
        startRecorderMutation.mutate(
          {
            conversation_id: conversation.lastInsertId,
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
    <>
      <div className="flex flex-col sm:gap-4 sm:py-4 sm:pl-14">
        <header className="sticky top-0 z-30 flex h-14 items-center justify-between gap-4 border-b bg-background px-4 sm:static sm:h-auto sm:border-0 sm:bg-transparent sm:px-6">
          {pathname !== "/tray" && pathname !== "/main" && (
            <Link href={backPath}>
              <ChevronLeft className="h-5 w-5" />
            </Link>
          )}
          <div className="flex-grow"></div>
          <p className="absolute left-1/2 transform -translate-x-1/2">Platy</p>
        </header>
        <div className="p-2 h-screen flex flex-col gap-4">
          <Select
            value={selectedAudioInputDevice}
            onValueChange={(value) => setSelectedAudioInputDevice(value)}
          >
            <SelectTrigger className="w-full">
              <SelectValue placeholder={selectedAudioInputDevice} />
            </SelectTrigger>
            <SelectContent>
              {audioInputDevices.data?.map((device) => (
                <SelectItem value={device}>{device}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select
            value={selectedAudioOutputDevice}
            onValueChange={(value) => {
              invoke("set_target_output_device", { device: value });
              setSelectedAudioOutputDevice(value);
            }}
          >
            <SelectTrigger className="w-full">
              <SelectValue placeholder={selectedAudioOutputDevice} />
            </SelectTrigger>
            <SelectContent>
              {audioOutputDevices.data?.map((device) => (
                <SelectItem value={device}>{device}</SelectItem>
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
            {activeRecordingInfo?.status === "stopping" && (
              <Loader className="animate-spin" />
            )}
            {activeRecordingInfo?.status === "recording" && (
              <Circle
                className={clsx("text-red-800 fill-red-800 animate-pulse")}
              />
            )}
            {!activeRecordingInfo && (
              <Circle className={clsx("text-red-800")} />
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
                <TableCaption>
                  A list of your recent conversations.
                </TableCaption>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[100px]">Created at</TableHead>
                    <TableHead className="w-[100px]">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {conversations.data?.map((conversation) => (
                    <TableRow key={conversation.id}>
                      <TableCell className="font-medium">
                        {new Date(conversation.created_at).toLocaleDateString()}
                      </TableCell>
                      <TableCell className="font-medium flex justify-between">
                        <Link href={`/conversations/${conversation.id}`}>
                          <Button size={"sm"} variant={"secondary"}>
                            <Eye />
                          </Button>
                        </Link>

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
      </div>
    </>
  );
}
