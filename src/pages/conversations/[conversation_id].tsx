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

export default function Page() {
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

  return (
    <div className="p-2 h-screen flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg text-center">
            Conversation{" "}
            {new Date(conversation?.data?.created_at).toLocaleDateString()}
          </CardTitle>
        </CardHeader>
        <CardContent className="flex-1 overflow-y-scroll">
          {completeTranscription.data?.full_text.map((transcription) => {
            return transcription;
          })}
        </CardContent>
        <CardFooter className="flex flex-col gap-4"></CardFooter>
      </Card>
    </div>
  );
}
