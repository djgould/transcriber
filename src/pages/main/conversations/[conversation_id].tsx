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
import { invoke } from "@tauri-apps/api/core";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Checkbox } from "@/components/ui/checkbox";

const Page: NextPageWithLayout = () => {
  const {
    query: { conversation_id },
  } = useRouter();
  const conversation = useConversation(Number(conversation_id));

  const completeTranscription = useCompleteTranscription(
    Number(conversation_id)
  );

  const summarize = () => {
    invoke("summarize");
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
  const action_items = [
    {
      description:
        "Explore reinforcement learning as a means to enable AI models to go beyond their current capabilities",
      owner: "",
    },
    {
      description:
        "Investigate the use of self-play in AI models to develop creative moves",
      owner: "",
    },
    {
      description:
        "Research the potential for neural networks to learn from poorly labeled data and make better decisions than their training data",
      owner: "",
    },
    {
      description:
        "Develop approaches to add reasoning to AI models, such as adding heuristics on top of the model or allowing the model itself to develop reasoning as it scales up",
      owner: "",
    },
    {
      description:
        "Experiment with multimodality (images, video, sound) to enable AI models to make analogies and understand spatial things",
      owner: "",
    },
  ];
  const summary = `The speakers are discussing how AI models can go beyond their current capabilities and make creative moves or discoveries. They mention AlphaGo's move 37 as an example where experts initially thought it was a mistake, but later realized it was a brilliant move.
  
One speaker suggests that reinforcement learning, like AlphaGo used, may be the missing component for these models to go beyond their training data. The other speaker disagrees, pointing out that neural networks can learn from poorly labeled data and make better decisions than their training data.
  
The speakers discuss how these models will add reasoning in the future. One approach is to add heuristics on top of the model, while another way would be for the model itself to develop reasoning as it scales up.
  
The speaker suggests that as we scale up these models, they will get better at reasoning and making creative moves. They also mention AlphaGo's use of an evaluation function that was trained using Monte Carlo roll-out, which allowed it to revise its own intuitions about what the next move should be.
  
Finally, the speakers touch on multimodality, discussing how introducing images, video, and sound will change the models' ability to make analogies. They think it will make the models better at understanding spatial things and enable them to discover new analogies beyond human comprehension.
    `;
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
              <div className="py-2 flex flex-col gap-2">
                {action_items.map((actionItem) => (
                  <div className="flex items-center space-x-2">
                    <Checkbox id="terms" />
                    <label
                      htmlFor="terms"
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      {actionItem.description}
                    </label>
                  </div>
                ))}
              </div>
              <h4 className="scroll-m-20 text-xl font-semibold tracking-tight">
                Summary
              </h4>
              <p className="whitespace-pre-wrap">{summary}</p>
            </TabsContent>
            <TabsContent value="password">
              <Table>
                <TableBody>
                  {completeTranscription.data?.full_text.map(
                    (transcription, i) => {
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
