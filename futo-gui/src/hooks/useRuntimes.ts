import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface RuntimeEntry {
  runtime: string;
  version: string;
  status: string;
  path?: string;
  port?: number;
  memory_mb?: number;
  version_dir?: string | null;
  process?: string | null;
}

export function useRuntimes() {
  const [runtimes, setRuntimes] = useState<RuntimeEntry[]>([]);

  const fetchRuntimes = useCallback(async () => {
    try {
      const res = await invoke<string>("runtime_list");
      const data = JSON.parse(res);
      if (data.result?.installed) setRuntimes(data.result.installed);
    } catch { /* daemon offline */ }
  }, []);

  return { runtimes, fetchRuntimes };
}
