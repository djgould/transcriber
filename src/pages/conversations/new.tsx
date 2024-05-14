import { Link } from "@/components/catalyst-ui/link";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Circle, Speaker } from "lucide-react";
import { useRouter } from "next/router";
import { Slider } from "@/components/ui/slider";

const transcript = [
  {
    speaker: "Speaker 1",
    text: "Houston, we've had a problem here.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 2",
    text: "This is Houston. Say again please.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 1",
    text: "Uh Houston we've had a problem. We've had a main beam plus one volt.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 2",
    text: "Roger main beam interval.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 1",
    text: "Uh uh uh",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 2",
    text: "Speaker 2 So okay stand, by thirteen we're looking at it.",
    timeStamp: 0,
  },
  {
    speaker: "Speaker 1",
    text: " Okay uh right now uh Houston the uh voltage is uh is looking good um And we had a apretty large bank or so.",
    timeStamp: 0,
  },
];

export default function Page() {
  return (
    <div className="p-2">
      <Link href="/">Back</Link>
      <Card>
        <CardHeader>
          <CardTitle>Your Converstation</CardTitle>
          <CardDescription>Card Description</CardDescription>
        </CardHeader>
        <CardContent>
          {transcript.map((slice) => (
            <div className="mt-2 flex flex-col">
              <p>{slice.speaker}</p> <p>{slice.text}</p>
            </div>
          ))}
        </CardContent>
        <CardFooter className="flex flex-col gap-4">
          <Slider value={[50, 0]} />
          <Button variant="outline">
            <Circle className="text-red-800" />
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
