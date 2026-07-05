import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useTranslation } from "../i18n";
import {
  RefreshCw,
  MoreVertical,
  Settings,
  FolderOpen,
  Trash2,
  Play,
  Square,
  FolderSearch,
} from "lucide-react";
import type { RuntimeEntry } from "../hooks/useRuntimes";

const PORTS: Record<string, number> = {
  mariadb: 3306,
  mysql: 3306,
  postgresql: 5432,
  redis: 6379,
  apache: 80,
  nginx: 80,
};

const DB_RUNTIMES = new Set(["mariadb", "postgresql", "mysql"]);
const SERVER_RUNTIMES = new Set(["mariadb", "postgresql", "mysql", "apache", "nginx"]);
const WEB_SERVER_RUNTIMES = new Set(["apache", "nginx"]);

function loadDocRoot(runtime: string): string {
  try {
    return localStorage.getItem(`futou_docroot_${runtime}`) ?? "";
  } catch {
    return "";
  }
}

function saveDocRoot(runtime: string, path: string) {
  try {
    localStorage.setItem(`futou_docroot_${runtime}`, path);
  } catch {
    /* ignore */
  }
}

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
  runtime: r,
  installedVersions,
  isSelected,
  isMenuOpen,
  onToggleDone,
  onVersionSwitchDone,
  onSelectLog,
  onMenuToggle,
  onUninstall,
}: Props) {
  const port = r.port ?? PORTS[r.runtime];
  const isActive = r.status === "active";
  const isErr = r.status === "error";
  const isRunning = r.process === "running";
  const isServer = SERVER_RUNTIMES.has(r.runtime);
  const isWebServer = WEB_SERVER_RUNTIMES.has(r.runtime);
  const showDropdown = installedVersions.length > 1;
  const [configPath, setConfigPath] = useState<string | null>(null);
  const [docRootPrompt, setDocRootPrompt] = useState(false);
  const [docRoot, setDocRoot] = useState(() => loadDocRoot(r.runtime));
  const { t } = useTranslation();

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
        await invoke("runtime_activate", {
          runtime: r.runtime,
          version: r.version,
        });
      }
    } catch (e) {
      toast.error(t(on ? "error.activate" : "error.deactivate", { runtime: r.runtime, error: String(e) }));
    }
    onToggleDone();
  }

  async function doStart() {
    if (isWebServer) {
      setDocRootPrompt(true);
      return;
    }
    await startServer(undefined);
  }

  async function startServer(overrideDocRoot?: string) {
    const root = overrideDocRoot ?? docRoot;
    if (isWebServer && !root) return;
    try {
      await invoke("runtime_start", {
        runtime: r.runtime,
        version: r.version,
        ...(isWebServer && root ? { documentRoot: root } : {}),
      });
      if (root) saveDocRoot(r.runtime, root);
      setDocRootPrompt(false);
    } catch (e) {
      toast.error(t("error.start", { runtime: r.runtime, error: String(e) }));
    }
    onToggleDone();
  }

  async function doStop() {
    try {
      await invoke("runtime_stop", { runtime: r.runtime });
    } catch (e) {
      toast.error(t("error.stop", { runtime: r.runtime, error: String(e) }));
    }
    onToggleDone();
  }

  async function doSwitch(newVersion: string) {
    if (newVersion === r.version) return;
    if (isRunning) {
      const isDb = DB_RUNTIMES.has(r.runtime);
      const question = t(isDb ? "confirm.switchDb" : "confirm.switch", {
        runtime: r.runtime,
        from: r.version,
        to: newVersion,
        runtimeUpper: r.runtime.toUpperCase(),
      });
      if (!window.confirm(question)) return;
    }
    try {
      await invoke("runtime_deactivate", { runtime: r.runtime });
      await invoke("runtime_activate", {
        runtime: r.runtime,
        version: newVersion,
      });
    } catch (e) {
      toast.error(t("error.switch", { runtime: r.runtime, error: String(e) }));
    }
    onVersionSwitchDone();
  }

  async function handleBrowse() {
    try {
      const selected = await invoke<string>("pick_dir");
      if (selected) setDocRoot(selected);
    } catch {
      /* cancelled */
    }
  }

  return (
    <>
      <div
        className={`flex items-center gap-2.5 px-4 py-3 border-b border-futou-border relative cursor-pointer transition-colors last:border-b-0 hover:bg-futou-muted ${
          isSelected ? "bg-futou-accent-bg" : ""
        }`}
        onClick={(e) => {
          if ((e.target as HTMLElement).closest("button, select, input"))
            return;
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

        <span
          className={`w-2 h-2 rounded-full shrink-0 ${
            isErr
              ? "bg-futou-danger"
              : isRunning
                ? "bg-futou-success"
                : "bg-futou-text-muted"
          }`}
        />

        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm">{r.runtime}</div>
          <div
            className={`text-xs mt-0.5 ${
              isErr ? "text-futou-danger" : "text-futou-text-secondary"
            }`}
          >
            {isErr
              ? "Gagal start, lihat log"
              : isServer && port
                ? `Port ${port}`
                : ""}
          </div>
        </div>

        {showDropdown ? (
          <select
            value={r.version}
            onChange={(e) => doSwitch(e.target.value)}
            className="font-sans text-xs px-2 py-[5px] rounded-lg border border-futou-border-strong bg-futou-surface w-[88px] text-futou-text"
          >
            {installedVersions.map((v) => (
              <option key={v} value={v}>
                {v}
              </option>
            ))}
          </select>
        ) : (
          <span className="text-xs text-futou-text-secondary w-[88px] text-center shrink-0">
            {r.version}
          </span>
        )}

        <span className="text-xs text-futou-text-secondary w-12 text-right shrink-0">
          {r.memory_mb ?? 0} MB
        </span>

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
            onClick={() => (isRunning ? doStop() : doStart())}
          >
            {isRunning ? (
              <>
                <Square size={13} /> Stop
              </>
            ) : (
              <>
                <Play size={13} /> Start
              </>
            )}
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
            onClick={(e) => {
              e.stopPropagation();
              onMenuToggle();
            }}
          >
            <MoreVertical size={14} />
          </button>
          {isMenuOpen && (
            <div className="absolute right-0 top-full bg-futou-surface border border-futou-border-strong rounded-lg shadow-lg min-w-[160px] z-10 overflow-hidden">
              <button
                className={`flex w-full border-none rounded-none justify-start px-3 py-[9px] bg-transparent font-sans text-[13px] items-center gap-2 ${
                  configPath
                    ? "cursor-pointer text-futou-text hover:bg-futou-muted"
                    : "opacity-40 cursor-not-allowed text-futou-text-muted"
                }`}
                disabled={!configPath}
                title={
                  configPath ? `Edit ${configPath}` : "No config file found"
                }
                onClick={async () => {
                  if (configPath) {
                    try {
                      await invoke("open_file", { path: configPath });
                    } catch (e) {
                      toast.error(t("error.open", { error: String(e) }));
                    }
                  }
                  onMenuToggle();
                }}
              >
                <Settings size={14} />
                Config
              </button>
              <button
                className="flex w-full border-none rounded-none justify-start px-3 py-[9px] bg-transparent font-sans text-[13px] cursor-pointer text-futou-text items-center gap-2 hover:bg-futou-muted"
                onClick={async () => {
                  const dir = r.version_dir ?? r.path;
                  try {
                    await invoke("open_dir", { path: dir });
                  } catch (e) {
                    toast.error(t("error.open", { error: String(e) }));
                  }
                  onMenuToggle();
                }}
              >
                <FolderOpen size={14} />
                Open home dir
              </button>
              <button
                className="flex w-full border-none rounded-none justify-start px-3 py-[9px] bg-transparent font-sans text-[13px] cursor-pointer text-futou-danger items-center gap-2 hover:bg-futou-muted"
                onClick={onUninstall}
              >
                <Trash2 size={14} />
                Uninstall
              </button>
            </div>
          )}
        </div>
      </div>

      {docRootPrompt && (
        <div className="flex items-center gap-2 px-4 py-2 border-b border-futou-border bg-futou-muted">
          <FolderSearch size={15} className="text-futou-text-secondary shrink-0" />
          <input
            type="text"
            value={docRoot}
            onChange={(e) => setDocRoot(e.target.value)}
            placeholder="Document root path..."
            className="flex-1 border border-futou-border rounded-lg px-2 py-1 text-xs bg-white focus:outline-none focus:border-futou-accent"
            onKeyDown={(e) => {
              if (e.key === "Enter") startServer();
            }}
          />
          <button
            onClick={handleBrowse}
            className="text-xs border border-futou-border rounded-lg px-2 py-1 cursor-pointer hover:bg-futou-surface active:scale-95 whitespace-nowrap"
          >
            Browse
          </button>
          <button
            onClick={() => startServer()}
            className="text-xs bg-futou-accent text-white rounded-lg px-3 py-1 cursor-pointer hover:opacity-90 active:scale-95 whitespace-nowrap"
          >
            Start
          </button>
          <button
            onClick={() => setDocRootPrompt(false)}
            className="text-xs text-futou-text-secondary cursor-pointer hover:text-futou-text px-1"
          >
            ✕
          </button>
        </div>
      )}
    </>
  );
}
