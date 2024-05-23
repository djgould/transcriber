import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useToast } from "@/components/ui/use-toast";
import { invoke } from "@tauri-apps/api/core";

interface Conversation {
  id: number;
  created_at: string;
  updated_at: string;
}

export const useConversation = (conversationId: number) => {
  const conversation = useQuery({
    queryKey: ["conversations", conversationId],
    queryFn: async () => {
      const conversation = invoke("get_conversation", { conversationId });

      return conversation;
    },
  });

  return conversation;
};

export const useConversations = () => {
  console.log("use conversations");
  const conversations = useQuery({
    queryKey: ["conversations"],
    queryFn: async () => {
      const conversations = await invoke("get_conversations");
      console.log(conversations);
      return conversations;
    },
  });

  return conversations;
};

export const useCreateConversationMutation = () => {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  const createConversationMutation = useMutation({
    mutationFn: async () => {
      const conversation = await invoke("create_conversation", {
        form: { title: "the title" },
      });
      return conversation;
    },
    onError(error) {
      console.log(error);
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
  const { toast } = useToast();

  const deleteConversationMutation = useMutation({
    mutationFn: async ({ conversationId }: { conversationId: number }) => {
      return invoke("delete_conversation", { conversationId });
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
