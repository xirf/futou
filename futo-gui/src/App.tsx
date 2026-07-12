import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast, Toaster } from "sonner";
import { I18nProvider, useTranslation } from "./i18n";
import { useRuntimes } from "./hooks/useRuntimes";
import { useCatalogue } from "./hooks/useCatalogue";
import { useInstall } from "./hooks/useInstall";
import { Header } from "./components/Header";
import { ServiceList } from "./components/ServiceList";
import { LogPanel } from "./components/LogPanel";
import { AddServiceModal } from "./components/AddServiceModal";
import { SettingsPanel } from "./components/SettingsPanel";
import { WindowControls } from "./components/WindowControls";

function App() {
  const { t } = useTranslation();
  const { runtimes, fetchRuntimes } = useRuntimes();
  const { catalogue, fetchCatalogue } = useCatalogue();

  const [modalOpen, setModalOpen] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [selectedLog, setSelectedLog] = useState<string | null>(null);
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const [daemonRunning, setDaemonRunning] = useState(false);

  const onInstallDone = useCallback(() => {
    setModalOpen(false);
    fetchRuntimes();
  }, [fetchRuntimes]);

  const { installing, progress, startInstall } = useInstall(onInstallDone);

  async function checkDaemon() {
    try {
      await invoke("daemon_status");
      setDaemonRunning(true);
    } catch {
      setDaemonRunning(false);
    }
  }

  async function toggleDaemon() {
    if (daemonRunning) {
      try {
        await invoke("daemon_shutdown");
      } catch {
        /* gone already */
      }
    } else {
      await invoke("daemon_start");
      for (let i = 0; i < 10; i++) {
        await new Promise((r) => setTimeout(r, 400));
        try {
          await invoke("daemon_status");
          setDaemonRunning(true);
          await Promise.all([fetchRuntimes(), fetchCatalogue()]);
          return;
        } catch {
          /* still starting */
        }
      }
      toast.error(t("daemon.failedStart"));
    }
    await new Promise((r) => setTimeout(r, 500));
    checkDaemon();
  }

  useEffect(() => {
    let cancelled = false;

    async function startAndLoad() {
      try {
        await invoke("daemon_status");
      } catch {
        try {
          await invoke("daemon_start");
        } catch {
          if (!cancelled) toast.error(t("daemon.failedStart"));
          return;
        }
      }

      for (let i = 0; i < 10 && !cancelled; i++) {
        try {
          await invoke("daemon_status");
          setDaemonRunning(true);
          await Promise.all([fetchRuntimes(), fetchCatalogue()]);
          return;
        } catch {
          await new Promise((resolve) => setTimeout(resolve, 400));
        }
      }
      if (!cancelled) toast.error(t("daemon.failedStart"));
    }

    startAndLoad();
    return () => {
      cancelled = true;
    };
  }, [fetchRuntimes, fetchCatalogue, t]);

  useEffect(() => {
    const interval = setInterval(checkDaemon, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (!(e.target as HTMLElement).closest(".options-menu")) {
        setOpenMenu(null);
      }
    }
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, []);

  async function handleUninstall(runtime: string, version: string) {
    if (
      !window.confirm(
        t("confirm.uninstall", { runtime, version }),
      )
    )
      return;
    await invoke("runtime_uninstall", { runtime, version });
    fetchRuntimes();
    setOpenMenu(null);
  }

  if (showSettings) {
    return (
      <div className="max-w-[760px] mx-auto pb-[60px] pt-8">
        <div
          data-tauri-drag-region
          className="fixed top-0 left-0 right-0 h-8 z-40"
        />
        <WindowControls />
        <SettingsPanel onClose={() => setShowSettings(false)} />
      </div>
    );
  }

  const activeCount = runtimes.filter((r) => r.status === "active").length;

  return (
    <div className="max-w-[760px] mx-auto px-5 pb-[60px] pt-8">
      <div
        data-tauri-drag-region
        className="fixed top-0 left-0 right-0 h-8 z-40"
      />
      <WindowControls />

      <Header
        count={runtimes.length}
        activeCount={activeCount}
        daemonRunning={daemonRunning}
        onAdd={() => setModalOpen(true)}
        onToggleDaemon={toggleDaemon}
        onSettings={() => setShowSettings(true)}
      />

      <ServiceList
        runtimes={runtimes}
        selectedLog={selectedLog}
        openMenu={openMenu}
        onToggleDone={fetchRuntimes}
        onVersionSwitchDone={fetchRuntimes}
        onSelectLog={setSelectedLog}
        onMenuToggle={setOpenMenu}
        onUninstall={handleUninstall}
      />

      <LogPanel selectedLog={selectedLog} />

      <AddServiceModal
        open={modalOpen}
        catalogue={catalogue}
        installed={runtimes}
        installing={installing}
        installProgress={progress}
        onClose={() => setModalOpen(false)}
        onInstall={startInstall}
      />
    </div>
  );
}

export default function WrappedApp() {
  return (
    <I18nProvider>
      <Toaster position="bottom-right" richColors closeButton />
      <App />
    </I18nProvider>
  );
}
