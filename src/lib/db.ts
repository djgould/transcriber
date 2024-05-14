import Database from "tauri-plugin-sql-api";

export async function getDb() {
  const db = await Database.load("sqlite:test.db");
  return db;
}
