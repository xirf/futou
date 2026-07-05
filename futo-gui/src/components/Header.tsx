import { Plus, Power, Settings } from "lucide-react";
import { useTranslation } from "../i18n";

interface Props {
  count: number;
  activeCount: number;
  daemonRunning: boolean;
  onAdd: () => void;
  onToggleDaemon: () => void;
  onSettings: () => void;
}

export function Header({
  count,
  activeCount,
  daemonRunning,
  onAdd,
  onToggleDaemon,
  onSettings,
}: Props) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center justify-between mb-5 px-0">
      <div>
        <h1 className="text-lg font-semibold m-0">{t("app.title")}</h1>
        <p className="text-[13px] text-futou-text-secondary mt-0.5">
          {t("app.subtitle", { count, activeCount })} &middot; Daemon{" "}
          <span
            className={
              daemonRunning ? "text-futou-success" : "text-futou-danger"
            }
          >
            {daemonRunning ? t("app.daemonOn") : t("app.daemonOff")}
          </span>
        </p>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={onSettings}
          title={t("app.settings")}
          className="p-1.5 rounded-lg hover:bg-futou-muted cursor-pointer active:scale-95"
        >
          <Settings size={18} />
        </button>
        <button
          onClick={onToggleDaemon}
          title={daemonRunning ? "Stop daemon" : "Start daemon"}
          className={`inline-flex items-center gap-1 leading-none whitespace-nowrap font-sans text-[13px] border rounded-lg px-2.5 py-1.5 cursor-pointer active:scale-[0.98] ${
            daemonRunning
              ? "bg-futou-success-bg border-futou-success text-futou-success hover:bg-futou-success hover:text-white"
              : "bg-futou-danger-bg border-futou-danger text-futou-danger hover:bg-futou-danger hover:text-white"
          }`}
        >
          <Power size={14} />
        </button>
        <button
          onClick={onAdd}
          className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-text text-white border border-futou-text rounded-lg px-3 py-1.5 cursor-pointer active:scale-[0.98]"
        >
          <Plus size={16} />
          {t("app.add")}
        </button>
      </div>
    </div>
  );
}
