import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  selectedLog: string | null;
}

interface LogLine {
  timestamp: string;
  level: string;
  message: string;
}

export function LogPanel({ selectedLog }: Props) {
  const [logs, setLogs] = useState<LogLine[]>([]);

  useEffect(() => {
    if (!selectedLog) {
      setLogs([]);
      return;
    }

    async function fetch() {
      try {
        const res = await invoke<string>("runtime_logs", { runtime: selectedLog });
        const data = JSON.parse(res);
        if (data.result?.entries) setLogs(data.result.entries);
      } catch { /* daemon offline */ }
    }

    fetch();
    const interval = setInterval(fetch, 2000);
    return () => clearInterval(interval);
  }, [selectedLog]);

  return (
    <div className="mt-4">
      <p className="text-xs text-futou-text-muted mb-1.5">
        Log, {selectedLog ?? "-"}
      </p>
      <div className="bg-futou-muted rounded-lg p-2.5 font-mono text-xs text-futou-text-secondary max-h-[200px] overflow-y-auto">
        {!selectedLog ? (
          <div>[--:--:--] Select a service to view logs</div>
        ) : logs.length === 0 ? (
          <div>[--:--:--] No log entries yet</div>
        ) : (
          logs.map((entry, i) => (
            <div key={i} className={entry.level === "error" ? "text-futou-danger" : ""}>
              [{entry.timestamp}] {entry.message}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
