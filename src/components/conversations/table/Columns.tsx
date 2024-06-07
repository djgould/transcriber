"use client";

import { Button } from "@/components/ui/button";
import { useDeleteConversationMutation } from "@/hooks/useConversations";
import { ColumnDef } from "@tanstack/react-table";
import { Eye, Trash } from "lucide-react";
import Link from "next/link";

// This type is used to define the shape of our data.
// You can use a Zod schema here if you want.
export type Conversation = {
  id: number;
  created_at: string;
};

export const columns: ColumnDef<Conversation>[] = [
  {
    accessorKey: "created_at",
    header: "Created At",
    cell: ({ row }) => {
      return new Date(row.original.created_at).toLocaleDateString();
    },
  },
  {
    accessorKey: "actions",
    header: "Actions",
    cell: function ActionCell({ row }) {
      const deleteConversationMutation = useDeleteConversationMutation();
      return (
        <div className="flex gap-2">
          <Link href={`/main/conversations/${row.original.id}`}>
            <Button size={"sm"} variant={"secondary"}>
              <Eye />
            </Button>
          </Link>

          <Button
            size={"sm"}
            variant={"secondary"}
            onClick={() => {
              deleteConversationMutation.mutate({
                conversationId: row.original.id,
              });
            }}
          >
            <Trash />
          </Button>
        </div>
      );
    },
  },
];
