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
import { MainLayout } from "@/components/layout/main";
import { NextPageWithLayout } from "./_app";
import { DataTable } from "@/components/conversations/table/DataTable";
import { columns } from "@/components/conversations/table/Columns";

import "@blocknote/core/fonts/inter.css";
import { BlockNoteView } from "@blocknote/shadcn";
import "@blocknote/shadcn/style.css";
import { useCreateBlockNote } from "@blocknote/react";

const Page: NextPageWithLayout = () => {
  const editor = useCreateBlockNote();

  return (
    <Card className="h-screen">
      <CardContent className="flex-1 overflow-y-scroll pt-10">
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
