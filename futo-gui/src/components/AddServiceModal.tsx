import { useState, useMemo } from "react";
import { X, Database, Binary, Terminal } from "lucide-react";
import type { CatalogueEntry } from "../hooks/useCatalogue";
import type { RuntimeEntry } from "../hooks/useRuntimes";
import { CatalogueItem } from "./CatalogueItem";

type ProgressMap = Record<string, { progress: number; message: string; stage: string }>;

const ICONS: Record<string, { icon: typeof Database; desc: string }> = {
  php: { icon: Binary, desc: "PHP: Hypertext Preprocessor" },
  mariadb: { icon: Database, desc: "MariaDB database server" },
  postgresql: { icon: Database, desc: "PostgreSQL database server" },
  nginx: { icon: Terminal, desc: "Web server dan reverse proxy" },
  node: { icon: Binary, desc: "JavaScript runtime" },
  python: { icon: Terminal, desc: "Bahasa pemrograman umum" },
  redis: { icon: Terminal, desc: "In-memory data store" },
};

interface Props {
  open: boolean;
  catalogue: CatalogueEntry[];
  installed: RuntimeEntry[];
  installing: Set<string>;
  installProgress: ProgressMap;
  onClose: () => void;
  onInstall: (runtime: string, version: string) => void;
}

export function AddServiceModal({ open, catalogue, installed, installing, installProgress, onClose, onInstall }: Props) {
  const [search, setSearch] = useState("");

  const installedSet = useMemo(
    () => new Set(installed.map((r) => `${r.runtime}@${r.version}`)),
    [installed],
  );

  const filtered = catalogue
    .filter((e) => e.runtime.toLowerCase().includes(search.toLowerCase()))
    .map((entry) => ({
      ...entry,
      versions: entry.versions.filter((v) => !installedSet.has(`${entry.runtime}@${v}`)),
    }))
    .filter((entry) => entry.versions.length > 0);

  if (!open) return null;

  return (
    <div className="fixed inset-0 bg-black/35 flex items-start justify-center pt-20 z-50" onClick={onClose}>
      <div
        className="bg-futou-surface rounded-xl border border-futou-border w-[420px] max-w-[calc(100vw-40px)] p-[18px] max-h-[75vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-3">
          <p className="text-[15px] font-semibold">Add service</p>
          <button
            onClick={onClose}
            className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-surface border border-futou-border-strong rounded-lg px-2 py-1.5 cursor-pointer text-futou-text hover:bg-futou-muted active:scale-[0.98]"
          >
            <X size={16} />
          </button>
        </div>
        <input
          type="text"
          className="w-full px-2.5 py-2 rounded-lg border border-futou-border-strong font-sans text-[13px] mb-3 outline-none text-futou-text bg-futou-surface focus:border-futou-accent"
          placeholder="Cari service, mariadb, redis, nginx"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          autoFocus
        />
        <div className="overflow-y-auto flex-1 min-h-0">
          {filtered.map((entry) => {
            const meta = ICONS[entry.runtime] ?? { icon: Database, desc: entry.runtime };
            const key = `${entry.runtime}@${entry.versions[0]}`;
            const installedVer = entry.versions.find((v) => installing.has(`${entry.runtime}@${v}`));
            const isBusy = installedVer !== undefined;
            const prog = installedVer ? installProgress[`${entry.runtime}@${installedVer}`] : undefined;

            return (
              <CatalogueItem
                key={key}
                entry={entry}
                icon={meta.icon}
                desc={meta.desc}
                isInstalling={isBusy}
                progress={prog}
                onInstall={(v) => onInstall(entry.runtime, v)}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
}
