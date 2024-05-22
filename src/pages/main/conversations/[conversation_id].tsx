"use client";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { useConversation } from "@/hooks/useConversations";
import Link from "next/link";
import { useRouter } from "next/router";
import { Skeleton } from "@/components/ui/skeleton";
import { useCompleteTranscription } from "@/hooks/useTranscription";
import { MainLayout } from "@/components/layout/main";
import { ReactElement } from "react";
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

const Page: NextPageWithLayout = () => {
  const {
    query: { conversation_id },
  } = useRouter();
  const conversation = useConversation(Number(conversation_id));

  const completeTranscription = useCompleteTranscription(
    Number(conversation_id)
  );

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

  return (
    <div className="p-2 h-screen flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg text-center">
            Conversation {date.toLocaleString()}
          </CardTitle>
        </CardHeader>
        <CardContent className="flex-1 overflow-y-scroll">
          <Table>
            <TableBody>
              {completeTranscription.data?.full_text.map((transcription, i) => {
                return (
                  <TableRow>
                    <TableCell className="min-w-32">
                      {i % 2 == 0 ? "Speaker 1" : "Speaker 2"}
                    </TableCell>
                    <TableCell className="font-medium">
                      {" "}
                      {transcription}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
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
