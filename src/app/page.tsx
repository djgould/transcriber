"use client";
import { useEffect, useState } from "react";

import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../components/ui/table";
import Database from "tauri-plugin-sql-api";
import { createMeetingMutation, useMeetings } from "@/hooks/useMeetings";

function App() {
  const [isRecording, setIsRecording] = useState(false);
  const [greetMsg, setGreetMsg] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  const meetings = useMeetings();

  const meetingMutation = createMeetingMutation();

  useEffect(() => {
    async function loadDb() {
      console.log("loading db");
      try {
        const db = await Database.load("sqlite:test.db");
        console.log(await db.select("./tables"));
      } catch (e) {
        console.log(e);
      }
      console.log("loaded db");
    }

    loadDb();
  }, []);

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
            {loading ? "Loading..." : "Transcribeppp"}
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
              <TableCell className="font-medium">{meeting.name}</TableCell>
              <TableCell>{meeting.transription}</TableCell>
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
        <Button onClick={() => meetingMutation.mutate()}>Create Meeting</Button>
      </footer>
    </div>
  );
}

export default App;
