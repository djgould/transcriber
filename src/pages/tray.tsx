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
import {
  Calendar,
  ChevronLeft,
  ChevronRight,
  Circle,
  Eye,
  Loader,
  Trash,
  Trash2,
} from "lucide-react";
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
import { RecordingButton } from "@/components/recording/RecordingButton";
import { columns } from "@/components/conversations/table/Columns";
import { DataTable } from "@/components/conversations/table/DataTable";

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
  const [currentPage, setCurrentPage] = useState(1);

  const startRecorderMutation = useStartRecorderMutation();
  const stopRecorderMutation = useStopRecorderMutation();
  const isRecording = useIsRecording();

  const conversations = useConversations(currentPage, 3);
  const createConversationMutation = useCreateConversationMutation();
  const deleteConversationMutation = useDeleteConversationMutation();

  const viewConversation = async (conversationId: number) => {
    await invoke("open_conversation", { conversationId });
  };

  interface Conversation {
    id: number;
    created_at: string;
    updated_at: string;
  }

  return (
    <div className="w-[300px] min-h-screen bg-background text-foreground p-4 flex flex-col space-y-4 dark">
      {/* Audio Settings */}
      <Select
        value={selectedAudioInputDevice}
        onValueChange={(value) => {
          invoke("set_input_device_name", { name: value });
          setSelectedAudioInputDevice(value);
        }}
      >
        <SelectTrigger className="w-full bg-card text-sm">
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
        <SelectTrigger className="w-full bg-card text-sm">
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

      <div className="w-full h-[1px] bg-border my-2"></div>

      {/* Record Button */}
      <RecordingButton />

      {/* Upcoming Meetings */}
      <Card className="bg-card">
        <CardHeader className="py-2 px-4">
          <CardTitle className="text-sm font-semibold flex items-center gap-2">
            <Calendar className="h-4 w-4" />
            Upcoming Meetings
          </CardTitle>
        </CardHeader>
        <CardContent className="py-2 px-4">
          <div className="space-y-2 text-xs">
            <div className="flex justify-between items-center p-2 bg-muted rounded">
              <div>
                <h3 className="font-medium">Team Sync</h3>
                <p className="text-muted-foreground">Today, 2:00 PM</p>
              </div>
              <Button variant="ghost" size="sm" className="h-6 text-xs px-2">
                Join
              </Button>
            </div>
            <div className="flex justify-between items-center p-2 bg-muted rounded">
              <div>
                <h3 className="font-medium">Project Review</h3>
                <p className="text-muted-foreground">Tomorrow, 10:00 AM</p>
              </div>
              <Button variant="ghost" size="sm" className="h-6 text-xs px-2">
                Join
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Conversations */}
      <Card className="bg-card">
        <CardHeader className="py-2 px-4">
          <CardTitle className="text-sm font-semibold">
            Your Conversations
          </CardTitle>
        </CardHeader>
        <CardContent className="py-2 px-4">
          <div className="space-y-2 text-xs">
            {conversations.data?.conversations.map(
              (conversation: Conversation) => (
                <div
                  key={conversation.id}
                  className="flex justify-between items-center p-2 bg-muted rounded"
                >
                  <span>
                    {new Date(conversation.created_at).toLocaleDateString()}
                  </span>
                  <div className="flex gap-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 w-6 p-0"
                      onClick={() => viewConversation(conversation.id)}
                    >
                      <Eye className="h-3 w-3" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 w-6 p-0"
                      onClick={() =>
                        deleteConversationMutation.mutate({
                          conversationId: conversation.id,
                        })
                      }
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              )
            )}
          </div>
        </CardContent>
        <CardFooter className="flex justify-between pt-2 text-xs">
          <Button
            variant="ghost"
            size="sm"
            className="h-6 px-2 text-muted-foreground"
            onClick={() => setCurrentPage((prev) => Math.max(1, prev - 1))}
            disabled={currentPage === 1}
          >
            <ChevronLeft className="h-3 w-3 mr-1" />
            Previous
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 px-2"
            onClick={() => setCurrentPage((prev) => prev + 1)}
            disabled={
              !conversations.data || currentPage >= conversations.data.numPages
            }
          >
            Next
            <ChevronRight className="h-3 w-3 ml-1" />
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
};

Page.getLayout = function getLayout(page: ReactElement) {
  return <TrayLayout>{page}</TrayLayout>;
};

export default Page;
