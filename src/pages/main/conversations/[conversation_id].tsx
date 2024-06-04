"use client";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  useConversation,
  useConverstaionSummary,
} from "@/hooks/useConversations";
import Link from "next/link";
import { useRouter } from "next/router";
import { Skeleton } from "@/components/ui/skeleton";
import { useCompleteTranscription } from "@/hooks/useTranscription";
import { MainLayout } from "@/components/layout/main";
import { ReactElement, useState } from "react";
import { NextPageWithLayout } from "@/pages/_app";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { invoke } from "@tauri-apps/api/core";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Checkbox } from "@/components/ui/checkbox";
import { Check, Clipboard, User } from "lucide-react";
import Markdown from "react-markdown";
import copy from "copy-to-clipboard";

const Page: NextPageWithLayout = () => {
  const {
    query: { conversation_id },
  } = useRouter();
  const conversation = useConversation(Number(conversation_id));

  const completeTranscription = useCompleteTranscription(
    Number(conversation_id)
  );

  const conversationSummary = useConverstaionSummary(Number(conversation_id));
  const [copied, setCopied] = useState(false);

  const copyTranscription = () => {
    completeTranscription.data?.full_text &&
      copy(completeTranscription.data?.full_text.join("\n"));
    setCopied(true);

    setTimeout(() => {
      setCopied(false);
    }, 3000);
  };

  if (!conversation.data) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg text-center">
            <Skeleton className="w-[100px] h-[20px] rounded-full" />
          </CardTitle>
        </CardHeader>
        <CardContent className="flex-1 overflow-y-scroll">
          <Skeleton className="w-[100px] h-[20px] rounded-full" />
        </CardContent>
        <CardFooter className="flex flex-col gap-4">
          <Skeleton className="w-[100px] h-[20px] rounded-full" />
        </CardFooter>
      </Card>
    );
  }

  const date = new Date(conversation?.data?.created_at);
  const summary = conversationSummary.data?.result;
  return (
    <div className="p-2 h-screen flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg text-center">
            Conversation {date.toLocaleString()}
          </CardTitle>
        </CardHeader>
        <CardContent className="flex-1 overflow-y-scroll">
          <Tabs defaultValue="account" className="w-full">
            <TabsList>
              <TabsTrigger value="account">Summary</TabsTrigger>
              <TabsTrigger value="password">Transcript</TabsTrigger>
            </TabsList>
            <TabsContent value="account">
              <h4 className="scroll-m-20 text-xl font-semibold tracking-tight">
                Action Items
              </h4>
              <div className="flex flex-col gap-4 py-4">
                {conversationSummary.data?.action_items.map((actionItem) => (
                  <div
                    className="flex items-center space-x-2"
                    key={actionItem.title}
                  >
                    <Checkbox id="terms" />
                    <label
                      htmlFor="terms"
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      {actionItem.title}
                    </label>
                  </div>
                ))}
              </div>
              <h4 className="scroll-m-20 text-xl font-semibold tracking-tight">
                Summary
              </h4>
              <Markdown>{summary}</Markdown>
            </TabsContent>
            <TabsContent value="password" className="relative">
              <Button
                className="absolute -top-12 right-0"
                variant="secondary"
                onClick={copyTranscription}
              >
                {copied ? <Check /> : <Clipboard />}
              </Button>
              <Table>
                <TableBody>
                  {completeTranscription.data?.full_text.map(
                    (transcription, i) => {
                      return (
                        <TableRow key={`row-${i}`}>
                          <TableCell className="min-w-8">
                            <User />
                          </TableCell>
                          <TableCell className="font-medium">
                            {" "}
                            {transcription}
                          </TableCell>
                        </TableRow>
                      );
                    }
                  )}
                </TableBody>
              </Table>
            </TabsContent>
          </Tabs>
        </CardContent>
        <CardFooter className="flex flex-col gap-4"></CardFooter>
      </Card>
    </div>
  );
};

Page.getLayout = function getLayout(page: ReactElement) {
  return <MainLayout>{page}</MainLayout>;
};

export default Page;
