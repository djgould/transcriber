import { getDbInstance } from "@/lib/db";
import { useEffect, useRef, useState } from "react";
import Database from "tauri-plugin-sql-api";

export function useDb() {
  const [db, setDb] = useState<Database | null>(null);

  useEffect(() => {
    let isMounted = true;

    async function connectToDb() {
      if (isMounted) {
        const dbInstance = await getDbInstance();
        setDb(dbInstance);
      }
    }
    connectToDb();
    return () => {
      isMounted = false;
    };
  }, []);

  return db;
}

export class NotConnectedToDbError extends Error {
  constructor() {
    super("Not connected to db");
  }
}
