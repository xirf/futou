import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRuntimes } from "./hooks/useRuntimes";
import { useCatalogue } from "./hooks/useCatalogue";
import { useInstall } from "./hooks/useInstall";
import { Header } from "./components/Header";
import { ServiceList } from "./components/ServiceList";
import { LogPanel } from "./components/LogPanel";
import { AddServiceModal } from "./components/AddServiceModal";

function App() {
  const { runtimes, fetchRuntimes } = useRuntimes();
  const { catalogue, fetchCatalogue } = useCatalogue();

  const [modalOpen, setModalOpen] = useState(false);
  const [selectedLog, setSelectedLog] = useState<string | null>(null);
  const [openMenu, setOpenMenu] = useState<string | null>(null);

  const onInstallDone = useCallback(() => {
    setModalOpen(false);
    fetchRuntimes();
  }, [fetchRuntimes]);

  const { installing, progress, startInstall } = useInstall(onInstallDone);

  useEffect(() => {
    fetchRuntimes();
    fetchCatalogue();
  }, [fetchRuntimes, fetchCatalogue]);

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
    await invoke("runtime_uninstall", { runtime, version });
    fetchRuntimes();
    setOpenMenu(null);
  }

  const activeCount = runtimes.filter((r) => r.status === "active").length;

  return (
    <div className="max-w-[760px] mx-auto px-5 pb-[60px] pt-6">
      <Header count={runtimes.length} activeCount={activeCount} onAdd={() => setModalOpen(true)} />

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

export default App;
