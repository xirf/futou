import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ArrowLeft, FolderOpen, Save } from "lucide-react";
import { useTranslation } from "../i18n";

interface Props {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: Props) {
  const { t, lang, setLang } = useTranslation();
  const [installDir, setInstallDir] = useState("");
  const [autostart, setAutostart] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const config = await invoke<string>("get_config");
        const parsed = JSON.parse(config);
        if (parsed.install_dir) setInstallDir(parsed.install_dir);
      } catch {
        /* use default */
      }
      try {
        const a = await invoke<boolean>("get_autostart");
        setAutostart(a);
      } catch {
        /* use default */
      }
    })();
  }, []);

  async function handleBrowse() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select install directory",
    });
    if (selected) setInstallDir(selected as string);
  }

  async function handleSave() {
    try {
      await invoke("set_install_location", { path: installDir });
      await invoke("set_autostart", { enabled: autostart });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Save failed:", e);
    }
  }

  return (
    <div className="px-5">
      <div className="flex items-center gap-3 mb-6">
        <button
          onClick={onClose}
          className="p-1.5 rounded-lg hover:bg-futou-muted cursor-pointer active:scale-95"
          title={t("app.back")}
        >
          <ArrowLeft size={18} />
        </button>
        <div>
          <h2 className="text-lg font-semibold m-0">{t("settings.title")}</h2>
        </div>
      </div>

      <div className="space-y-6">
        <div>
          <label className="block text-sm font-medium mb-1.5">
            {t("settings.language")}
          </label>
          <select
            value={lang}
            onChange={(e) => setLang(e.target.value as "en" | "id")}
            className="w-full border border-futou-border rounded-lg px-3 py-2 text-sm bg-white cursor-pointer focus:outline-none focus:border-futou-accent"
          >
            <option value="en">{t("settings.languageEn")}</option>
            <option value="id">{t("settings.languageId")}</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1.5">
            {t("settings.installLocation")}
          </label>
          <p className="text-xs text-futou-text-secondary mb-2">
            {t("settings.installLocationDesc")}
          </p>
          <div className="flex gap-2">
            <input
              type="text"
              value={installDir}
              onChange={(e) => setInstallDir(e.target.value)}
              placeholder="C:\Users\...\.futou"
              className="flex-1 border border-futou-border rounded-lg px-3 py-2 text-sm bg-white focus:outline-none focus:border-futou-accent"
            />
            <button
              onClick={handleBrowse}
              className="inline-flex items-center gap-1 text-sm border border-futou-border rounded-lg px-3 py-2 cursor-pointer hover:bg-futou-muted active:scale-95"
            >
              <FolderOpen size={15} />
              {t("settings.browse")}
            </button>
          </div>
        </div>

        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">
              {t("settings.daemonAutostart")}
            </div>
            <div className="text-xs text-futou-text-secondary mt-0.5">
              {t("settings.daemonAutostartDesc")}
            </div>
          </div>
          <button
            onClick={() => setAutostart(!autostart)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors cursor-pointer ${
              autostart ? "bg-futou-success" : "bg-futou-border-strong"
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                autostart ? "translate-x-6" : "translate-x-1"
              }`}
            />
          </button>
        </div>
      </div>

      <div className="mt-8">
        <button
          onClick={handleSave}
          className="inline-flex items-center gap-1.5 leading-none font-sans text-sm bg-futou-accent text-white border-0 rounded-lg px-4 py-2 cursor-pointer active:scale-95 hover:opacity-90"
        >
          <Save size={15} />
          {saved ? t("app.saved") : t("app.save")}
        </button>
      </div>
    </div>
  );
}
