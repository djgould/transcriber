import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { NotConnectedToDbError, useDb } from "./useDb";
import { useToast } from "@/components/ui/use-toast";
import { invoke } from "@tauri-apps/api/core";

interface Conversation {
  id: number;
  created_at: string;
  updated_at: string;
}

export const useConversation = (conversationId: number) => {
  const db = useDb();

  const conversation = useQuery({
    queryKey: ["conversations", conversationId],
    queryFn: async () => {
      if (!db) throw new NotConnectedToDbError();
      const conversation = await db.select<Conversation[]>(
        "SELECT * FROM conversations WHERE id = ?",
        [conversationId]
      );
      if (conversation.length === 0) throw "Conversation not found";
      if (conversation.length > 1) throw "Multiple conversations found";
      return conversation[0];
    },
    enabled: !!db,
  });

  return conversation;
};

export const useConversations = () => {
  const db = useDb();

  const conversations = useQuery({
    queryKey: ["conversations"],
    queryFn: async () => {
      if (!db) throw new NotConnectedToDbError();
      const conversations = await db.select<Conversation[]>(
        "SELECT * FROM conversations"
      );
      return conversations;
    },
    enabled: !!db,
  });

  return conversations;
};

export const useCreateConversationMutation = () => {
  const queryClient = useQueryClient();
  const db = useDb();
  const { toast } = useToast();

  const createConversationMutation = useMutation({
    mutationFn: async () => {
      if (!db) throw new NotConnectedToDbError();
      const result = await db.execute(
        "INSERT INTO conversations DEFAULT VALUES;"
      );

      return result;
    },
    onError(error) {
      toast({
        title: "Error Creating Conversation",
        description: error.message,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["conversations"] });
      const { dismiss } = toast({
        title: "Created Conversation",
        description: "Successful",
      });

      setTimeout(() => {
        dismiss();
      }, 2000);
    },
  });

  return createConversationMutation;
};

export const useDeleteConversationMutation = () => {
  const queryClient = useQueryClient();
  const db = useDb();
  const { toast } = useToast();

  const deleteConversationMutation = useMutation({
    mutationFn: async ({ conversationId }: { conversationId: number }) => {
      if (!db) throw new NotConnectedToDbError();
      try {
        await invoke("delete_recording_data", { conversationId });
      } catch (error) {
        if (error !== "No such file or directory (os error 2)") {
          throw error;
        }
      }
      return db.execute("DELETE FROM conversations WHERE id = ?", [
        conversationId,
      ]);
    },
    onError(error) {
      toast({
        title: "Error Deleting Conversation",
        description: error,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["conversations"] });
      const { dismiss } = toast({
        title: "Deleted Conversation",
        description: "Successful",
      });
    },
  });

  return deleteConversationMutation;
};
