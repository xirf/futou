import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

type InstallProgress = { progress: number; message: string; stage: string };

export function useInstall(onDone: () => void) {
  const [installing, setInstalling] = useState<Set<string>>(new Set());
  const [progress, setProgress] = useState<Record<string, InstallProgress>>({});

  const startInstall = useCallback(async (runtime: string, version: string) => {
    const key = `${runtime}@${version}`;
    setInstalling((prev) => new Set(prev).add(key));

    try {
      const res = await invoke<string>("runtime_install", { runtime, version });
      const data = JSON.parse(res);
      const taskId = data.result?.task_id;
      if (!taskId) throw new Error("No task_id");

      const poll = setInterval(async () => {
        try {
          const pRes = await invoke<string>("runtime_install_status", { taskId });
          const pData = JSON.parse(pRes);
          const p = pData.result;
          if (!p) return;

          setProgress((prev) => ({ ...prev, [key]: { progress: p.progress, message: p.message, stage: p.stage } }));

          if (p.stage === "completed") {
            clearInterval(poll);
            setInstalling((prev) => { const n = new Set(prev); n.delete(key); return n; });
            setProgress((prev) => { const n = { ...prev }; delete n[key]; return n; });
            onDone();
          } else if (p.stage === "failed") {
            clearInterval(poll);
            setInstalling((prev) => { const n = new Set(prev); n.delete(key); return n; });
            setProgress((prev) => { const n = { ...prev }; delete n[key]; return n; });
            alert(`Install failed: ${p.message}`);
          }
        } catch { /* poll retry */ }
      }, 600);
    } catch (e) {
      setInstalling((prev) => { const n = new Set(prev); n.delete(key); return n; });
      alert(`Install error: ${e}`);
    }
  }, [onDone]);

  return { installing, progress, startInstall };
}
