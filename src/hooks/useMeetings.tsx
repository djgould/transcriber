import { getDb } from "@/lib/db";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

interface Meeting {
  name: string;
  transription: string[];
}

export function useMeetings() {
  const meetings = useQuery({
    queryKey: ["meetings"],
    queryFn: async () => {
      const db = await getDb();
      const _meetings = await db.select<Meeting[]>("SELECT * FROM meetings");
      console.log(_meetings);
      return _meetings;
    },
  });

  return meetings;
}

export function createMeetingMutation() {
  const queryClient = useQueryClient();
  const meetingMutation = useMutation({
    mutationKey: ["createMeeting"],
    mutationFn: async () => {
      console.log("mutation");
      const db = await getDb();
      console.log("got db");
      const _meeting = await db.execute(
        "INSERT INTO meetings (name, transcription) VALUES (?,?)",
        ["test", "test"]
      );
      console.log(_meeting);
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
