import { getDb } from "@/lib/db";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

interface RawMeeting {
  name: string;
  transcription: string;
  id: string;
  created_at: string;
  updated_at: string;
}
interface Meeting {
  name: string;
  transcription: string[];
  id: string;
  createdAt: Date;
  updatedAt: Date;
}

function parseMeeting(rawMeeting: RawMeeting): Meeting {
  return {
    id: rawMeeting.id,
    name: rawMeeting.name,
    transcription: JSON.parse(rawMeeting.transcription) as string[],
    createdAt: new Date(rawMeeting.created_at),
    updatedAt: new Date(rawMeeting.updated_at),
  };
}

export function useMeetings() {
  const meetings = useQuery({
    queryKey: ["meetings"],
    queryFn: async () => {
      const db = await getDb();
      const _meetings = await db.select<RawMeeting[]>("SELECT * FROM meetings");
      console.log(_meetings);
      return _meetings.map(parseMeeting);
    },
  });

  return meetings;
}

function meetingToJson(meeting: Meeting) {
  return {
    name: meeting.name,
    transcription: JSON.stringify(meeting.transcription),
  };
}

export function useCreateMeetingMutation() {
  const queryClient = useQueryClient();
  const meetingMutation = useMutation({
    mutationKey: ["createMeeting"],
    mutationFn: async (meeting: Meeting) => {
      const jsonMeeting = meetingToJson(meeting);
      console.log(jsonMeeting);
      const db = await getDb();
      const _meeting = await db.execute(
        "INSERT INTO meetings (name, transcription) VALUES (?,?)",
        [jsonMeeting.name, jsonMeeting.transcription]
      );
      return _meeting;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["meetings"],
      });
    },
    onError: (err) => {
      console.log(err);
    },
  });

  return meetingMutation;
}
