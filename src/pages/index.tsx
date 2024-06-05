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
import { RecordingButton } from "@/components/recording/RecordingButton";

export default function Page() {
  const audioInputDevices = useAudioInputDevicesQuery();
  const audioOutputDevices = useAudioOutputDevicesQuery();

  const [selectedAudioInputDevice, setSelectedAudioInputDevice] = useAtom(
    selectedAudioInputDeviceAtom
  );
  const [selectedAudioOutputDevice, setSelectedAudioOutputDevice] = useAtom(
    selectedAudioOutputDeviceAtom
  );

  const conversations = useConversations();
  const deleteConversationMutation = useDeleteConversationMutation();

  return (
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
            <SelectItem value={device} key={device}>
              {device}
            </SelectItem>
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
            <SelectItem value={device} key={device}>
              {device}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <RecordingButton />
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
  );
}
