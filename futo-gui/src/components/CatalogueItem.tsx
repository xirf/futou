import { Loader2 } from "lucide-react";
import type { CatalogueEntry } from "../hooks/useCatalogue";

type Progress = { progress: number; message: string; stage: string };

interface Props {
  entry: CatalogueEntry;
  icon: React.ComponentType<{ size?: number; className?: string }>;
  desc: string;
  isInstalling: boolean;
  progress?: Progress | undefined;
  onInstall: (version: string) => void;
}

export function CatalogueItem({ entry, icon: Icon, desc, isInstalling, progress, onInstall }: Props) {
  return (
    <div className={`flex items-center gap-3 p-2.5 border border-futou-border rounded-lg mb-2 ${isInstalling ? "bg-futou-accent-bg border-futou-accent-bg" : ""}`}>
      {isInstalling
        ? <Loader2 size={20} className={`shrink-0 ${isInstalling ? "text-futou-accent" : "text-futou-text-secondary"} animate-spin`} style={{ animation: "spin 1s linear infinite" }} />
        : <Icon size={20} className="shrink-0 text-futou-text-secondary" />
      }
      <div className="flex-1 min-w-0">
        <div className="font-medium text-[13px]">{entry.runtime}</div>
        <div className={`text-xs mt-px ${isInstalling ? "text-futou-accent" : "text-futou-text-secondary"}`}>
          {isInstalling
            ? progress ? `${progress.stage} ${progress.progress}%` : "Starting..."
            : desc
          }
        </div>
      </div>
      <select
        onClick={(e) => e.stopPropagation()}
        className="font-sans text-xs px-2 py-[5px] rounded-lg border border-futou-border-strong bg-futou-surface w-[88px] text-futou-text"
      >
        {entry.versions.map((v) => (
          <option key={v} value={v}>{v}</option>
        ))}
      </select>
      <button
        disabled={isInstalling}
        onClick={(e) => {
          e.stopPropagation();
          const sel = (e.currentTarget.parentElement!.querySelector("select")! as HTMLSelectElement).value;
          onInstall(sel);
        }}
        className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-surface border border-futou-border-strong rounded-lg px-3 py-1.5 cursor-pointer text-futou-text hover:bg-futou-muted active:scale-[0.98] disabled:opacity-40 disabled:cursor-not-allowed disabled:transform-none"
      >
        {isInstalling ? "Cancel" : "Install"}
      </button>
    </div>
  );
}
