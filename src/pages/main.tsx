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
  useStartRecorderMutation,
  useStopRecorderMutation,
} from "@/hooks/useRecorder";
import { Calendar, Circle, Eye, Loader, Menu, Mic, Trash } from "lucide-react";
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
import { MainLayout } from "@/components/layout/main";
import { NextPageWithLayout } from "./_app";
import { DataTable } from "@/components/conversations/table/DataTable";
import { columns } from "@/components/conversations/table/Columns";

import "@blocknote/core/fonts/inter.css";
import { BlockNoteView } from "@blocknote/shadcn";
import "@blocknote/shadcn/style.css";
import { useCreateBlockNote } from "@blocknote/react";

interface Meeting {
  title: string;
  time: string;
}

const Page: NextPageWithLayout = () => {
  const [isRecording, setIsRecording] = useState(false);
  const editor = useCreateBlockNote();

  const upcomingMeeting: Meeting = {
    title: "Team Sync",
    time: "2:00 PM",
  };

  return (
    <Card className="h-screen">
      <CardHeader>
        <div className="flex items-center justify-between">
          {/* Menu Toggle */}
          <Button
            variant="ghost"
            size="icon"
            className="text-white hover:bg-gray-800"
          >
            <Menu className="h-5 w-5" />
            <span className="sr-only">Toggle menu</span>
          </Button>

          <div className="flex items-center space-x-2">
            {/* Meeting Indicator */}
            {upcomingMeeting && (
              <Button
                variant="ghost"
                size="sm"
                className="hidden sm:flex items-center space-x-2 text-xs text-gray-300 hover:text-white hover:bg-gray-800"
              >
                <Calendar className="h-4 w-4" />
                <span>
                  {upcomingMeeting.title} • {upcomingMeeting.time}
                </span>
              </Button>
            )}

            {/* Recording Button */}
            <Button
              variant={isRecording ? "destructive" : "ghost"}
              size="icon"
              onClick={() => setIsRecording(!isRecording)}
              className={`transition-colors duration-200 ${
                isRecording
                  ? "bg-red-600 hover:bg-red-700 text-white"
                  : "text-white hover:bg-gray-800"
              }`}
            >
              <Mic
                className={`h-5 w-5 ${isRecording ? "animate-pulse" : ""}`}
              />
              <span className="sr-only">
                {isRecording ? "Stop recording" : "Start recording"}
              </span>
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="flex-1 overflow-y-scroll">
        <BlockNoteView editor={editor} className="h-full" />
        {/* <DataTable
            columns={columns}
            data={conversations.data || []}
            pageSize={8}
          /> */}
        {/* <Table>
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
                    <Link href={`/main/conversations/${conversation.id}`}>
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
          </Table> */}
      </CardContent>
    </Card>
  );
};

Page.getLayout = function getLayout(page: ReactElement) {
  return <>{page}</>;
};

export default Page;
