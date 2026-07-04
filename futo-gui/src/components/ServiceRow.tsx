import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, MoreVertical, Settings, FolderOpen, Trash2 } from "lucide-react";
import type { RuntimeEntry } from "../hooks/useRuntimes";

const PORTS: Record<string, number> = {
  mariadb: 3306, mysql: 3306, postgresql: 5432, redis: 6379,
};

const DB_RUNTIMES = new Set(["mariadb", "postgresql", "mysql"]);
const SERVER_RUNTIMES = new Set(["mariadb", "postgresql", "mysql"]);

interface Props {
  runtime: RuntimeEntry;
  installedVersions: string[];
  isSelected: boolean;
  isMenuOpen: boolean;
  onToggleDone: () => void;
  onVersionSwitchDone: () => void;
  onSelectLog: () => void;
  onMenuToggle: () => void;
  onUninstall: () => void;
}

export function ServiceRow({
  runtime: r, installedVersions, isSelected, isMenuOpen,
  onToggleDone, onVersionSwitchDone, onSelectLog, onMenuToggle, onUninstall,
}: Props) {
  const port = r.port ?? PORTS[r.runtime];
  const isActive = r.status === "active";
  const isErr = r.status === "error";
  const isRunning = r.process === "running";
  const isServer = SERVER_RUNTIMES.has(r.runtime);
  const showDropdown = installedVersions.length > 1;
  const [configPath, setConfigPath] = useState<string | null>(null);

  useEffect(() => {
    const vd = r.version_dir ?? r.path;
    if (!vd) return;
    invoke("find_config", { runtime: r.runtime, versionDir: vd })
      .then((path: unknown) => setConfigPath(path as string | null))
      .catch(() => setConfigPath(null));
  }, [r.runtime, r.version_dir, r.path]);

  async function doToggle(on: boolean) {
    try {
      if (!on) {
        await invoke("runtime_deactivate", { runtime: r.runtime });
      } else {
        await invoke("runtime_activate", { runtime: r.runtime, version: r.version });
      }
    } catch (e) {
      alert(`Gagal ${on ? "activate" : "deactivate"} ${r.runtime}: ${e}`);
    }
    onToggleDone();
  }

  async function doStart() {
    try {
      await invoke("runtime_start", { runtime: r.runtime, version: r.version });
    } catch (e) {
      alert(`Gagal start ${r.runtime}: ${e}`);
    }
    onToggleDone();
  }

  async function doStop() {
    try {
      await invoke("runtime_stop", { runtime: r.runtime });
    } catch (e) {
      alert(`Gagal stop ${r.runtime}: ${e}`);
    }
    onToggleDone();
  }

  async function doSwitch(newVersion: string) {
    if (newVersion === r.version) return;
    if (isRunning) {
      const isDb = DB_RUNTIMES.has(r.runtime);
      const question = isDb
        ? `Mengganti versi ${r.runtime} dari ${r.version} ke ${newVersion}.\n\nServer akan dihentikan.\n${r.runtime.toUpperCase()} adalah database — versi berbeda bisa menyebabkan ketidakcocokan data.\nLanjutkan?`
        : `Mengganti versi ${r.runtime} dari ${r.version} ke ${newVersion}.\n\nServer akan dihentikan dan diaktifkan kembali dengan versi baru.\nLanjutkan?`;
      if (!window.confirm(question)) return;
    }
    try {
      await invoke("runtime_deactivate", { runtime: r.runtime });
      await invoke("runtime_activate", { runtime: r.runtime, version: newVersion });
    } catch (e) {
      alert(`Gagal mengganti versi ${r.runtime}: ${e}`);
    }
    onVersionSwitchDone();
  }

  return (
    <div
      className={`flex items-center gap-2.5 px-4 py-3 border-b border-futou-border relative cursor-pointer transition-colors last:border-b-0 hover:bg-futou-muted ${isSelected ? "bg-futou-accent-bg" : ""}`}
      onClick={(e) => {
        if ((e.target as HTMLElement).closest("button, select, input")) return;
        onSelectLog();
      }}
    >
      <input
        type="checkbox"
        checked={isActive}
        onChange={() => doToggle(!isActive)}
        className="w-[15px] h-[15px] shrink-0 accent-futou-text"
        title="Add to PATH"
      />

      <span className={`w-2 h-2 rounded-full shrink-0 ${isErr ? "bg-futou-danger" : isRunning ? "bg-futou-success" : "bg-futou-text-muted"}`} />

      <div className="flex-1 min-w-0">
        <div className="font-medium text-sm">{r.runtime}</div>
        <div className={`text-xs mt-0.5 ${isErr ? "text-futou-danger" : "text-futou-text-secondary"}`}>
          {isErr ? "Gagal start, lihat log" : isServer && port ? `Port ${port}` : ""}
        </div>
      </div>

      {showDropdown ? (
        <select
          value={r.version}
          onChange={(e) => doSwitch(e.target.value)}
          className="font-sans text-xs px-2 py-[5px] rounded-lg border border-futou-border-strong bg-futou-surface w-[88px] text-futou-text"
        >
          {installedVersions.map((v) => (
            <option key={v} value={v}>{v}</option>
          ))}
        </select>
      ) : (
        <span className="text-xs text-futou-text-secondary w-[88px] text-center shrink-0">{r.version}</span>
      )}

      <span className="text-xs text-futou-text-secondary w-12 text-right shrink-0">{r.memory_mb ?? 0} MB</span>

      <button
        className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-surface border border-futou-border-strong rounded-lg px-2 py-1.5 cursor-pointer text-futou-text hover:bg-futou-muted active:scale-[0.98] disabled:opacity-40 disabled:cursor-not-allowed disabled:transform-none"
        title="Restart"
        disabled={!isRunning}
      >
        <RefreshCw size={14} />
      </button>

      {isServer ? (
        <button
          className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-surface border border-futou-border-strong rounded-lg px-3 py-1.5 cursor-pointer text-futou-text hover:bg-futou-muted active:scale-[0.98]"
          onClick={() => isRunning ? doStop() : doStart()}
        >
          {isRunning ? "Stop" : "Start"}
        </button>
      ) : (
        <button
          className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-surface border border-futou-border-strong rounded-lg px-3 py-1.5 cursor-pointer text-futou-text disabled:opacity-40 disabled:cursor-not-allowed disabled:transform-none opacity-40 cursor-not-allowed"
          disabled
          title="Not a server runtime"
        >
          Start
        </button>
      )}

      <div className="relative">
        <button
          className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-surface border border-futou-border-strong rounded-lg px-2 py-1.5 cursor-pointer text-futou-text hover:bg-futou-muted active:scale-[0.98]"
          aria-label="Options"
          onClick={(e) => { e.stopPropagation(); onMenuToggle(); }}
        >
          <MoreVertical size={14} />
        </button>
        {isMenuOpen && (
          <div className="absolute right-0 top-full bg-futou-surface border border-futou-border-strong rounded-lg shadow-lg min-w-[160px] z-10 overflow-hidden">
            <button
              className={`flex w-full border-none rounded-none justify-start px-3 py-[9px] bg-transparent font-sans text-[13px] items-center gap-2 ${configPath ? "cursor-pointer text-futou-text hover:bg-futou-muted" : "opacity-40 cursor-not-allowed text-futou-text-muted"}`}
              disabled={!configPath}
              title={configPath ? `Edit ${configPath}` : "No config file found"}
              onClick={async () => {
                if (configPath) {
                  try { await invoke("open_file", { path: configPath }); } catch (e) { alert(`${e}`); }
                }
                onMenuToggle();
              }}
            >
              <Settings size={14} />Config
            </button>
            <button
              className="flex w-full border-none rounded-none justify-start px-3 py-[9px] bg-transparent font-sans text-[13px] cursor-pointer text-futou-text items-center gap-2 hover:bg-futou-muted"
              onClick={async () => {
                const dir = r.version_dir ?? r.path;
                try { await invoke("open_dir", { path: dir }); } catch (e) { alert(`${e}`); }
                onMenuToggle();
              }}
            >
              <FolderOpen size={14} />Open home dir
            </button>
            <button
              className="flex w-full border-none rounded-none justify-start px-3 py-[9px] bg-transparent font-sans text-[13px] cursor-pointer text-futou-danger items-center gap-2 hover:bg-futou-muted"
              onClick={onUninstall}
            >
              <Trash2 size={14} />Uninstall
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
