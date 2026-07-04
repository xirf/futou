import type { RuntimeEntry } from "../hooks/useRuntimes";
import { ServiceRow } from "./ServiceRow";

interface Props {
  runtimes: RuntimeEntry[];
  selectedLog: string | null;
  openMenu: string | null;
  onToggleDone: () => void;
  onVersionSwitchDone: () => void;
  onSelectLog: (name: string) => void;
  onMenuToggle: (key: string | null) => void;
  onUninstall: (runtime: string, version: string) => void;
}

export function ServiceList({ runtimes, selectedLog, openMenu, onToggleDone, onVersionSwitchDone, onSelectLog, onMenuToggle, onUninstall }: Props) {
  if (runtimes.length === 0) {
    return (
      <div className="bg-futou-surface border border-futou-border rounded-lg">
        <div className="flex items-center justify-center py-3 px-4 text-futou-text-muted">
          No runtimes installed. Click Add to browse catalogue.
        </div>
      </div>
    );
  }

  return (
    <div className="bg-futou-surface border border-futou-border rounded-lg">
      {runtimes.map((r) => {
        const menuKey = `${r.runtime}-${r.version}`;
        const installedVersions = runtimes
          .filter((o) => o.runtime === r.runtime)
          .map((o) => o.version);

        return (
          <ServiceRow
            key={menuKey}
            runtime={r}
            installedVersions={installedVersions}
            isSelected={selectedLog === r.runtime}
            isMenuOpen={openMenu === menuKey}
            onToggleDone={onToggleDone}
            onVersionSwitchDone={onVersionSwitchDone}
            onSelectLog={() => onSelectLog(r.runtime)}
            onMenuToggle={() => onMenuToggle(openMenu === menuKey ? null : menuKey)}
            onUninstall={() => onUninstall(r.runtime, r.version)}
          />
        );
      })}
    </div>
  );
}
