"use client";
import { useEffect, useState } from "react";

import { invoke } from "@tauri-apps/api/core";
import { Button } from "../components/catalyst-ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../components/catalyst-ui/table";
import Database from "tauri-plugin-sql-api";
import { createMeetingMutation, useMeetings } from "@/hooks/useMeetings";
import { Link } from "@/components/catalyst-ui/link";

function App() {
  const [isRecording, setIsRecording] = useState(false);
  const [greetMsg, setGreetMsg] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  const meetings = useMeetings();

  const meetingMutation = createMeetingMutation();

  async function greet() {
    setLoading(true);
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    try {
      const value = (await invoke("transcribe", {
        path: "./samples/a13.wav",
      })) as string[];
      setGreetMsg(value);
    } finally {
      setLoading(false);
    }
  }

  async function recordTenSeconds() {
    await invoke("record");
  }

  async function startRecording() {
    setIsRecording(true);
    await invoke("start_recording").catch((err) => {
      console.error(err);
      setIsRecording(false);
    });
  }

  async function stopRecording() {
    setIsRecording(false);
    await invoke("stop_recording").catch((err) => {
      console.error(err);
    });
  }

  return (
    <div className="flex flex-col max-h-screen">
      <header className="h-16 flex justify-center p-4">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <Button disabled={loading} type="submit">
            {loading ? "Loading..." : "Transcribe"}
          </Button>
        </form>
      </header>
      <Table className="flex-grow flex-shrink overflow-y-scroll h-40">
        <TableHead>
          <TableRow>
            <TableHeader>Name</TableHeader>
            <TableHeader>Transcription</TableHeader>
          </TableRow>
        </TableHead>
        <TableBody>
          {meetings?.data?.map((meeting, i) => (
            <TableRow key={i}>
              <TableCell className="font-medium">
                <Link href={`/meetings/${meeting.id}`}>{meeting.name}</Link>
              </TableCell>
              <TableCell>{meeting.transcription}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
      <footer className="h-16 flex justify-center gap-4 p-4">
        <Button onClick={() => recordTenSeconds()}>Record 10 Seconds</Button>
        <Button
          onClick={() => (isRecording ? stopRecording() : startRecording())}
        >
          {isRecording ? "Stop Recording" : "Start Recording"}
        </Button>
        <Button onClick={() => setGreetMsg([])}>Clear</Button>
        <Link href="/meetings/new">Create Meeting</Link>
      </footer>
    </div>
  );
}

export default App;
