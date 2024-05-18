import Database from "tauri-plugin-sql-api";

let dbInstance: Database | null = null;

export async function getDbInstance() {
  if (!dbInstance) {
    dbInstance = await Database.load("sqlite:test.db");
    console.log("Database connected:", dbInstance);
  }
  return dbInstance;
}
