import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, Copy, X } from "lucide-react";

export function WindowControls() {
  const [maximized, setMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        setMaximized(await appWindow.isMaximized());
        unlisten = await appWindow.onResized(() => {
          appWindow.isMaximized().then(setMaximized);
        });
      } catch {
        /* not available in dev server */
      }
    })();
    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="fixed top-0 right-0 z-50 flex items-center">
      <button
        onClick={() => appWindow.minimize()}
        className="no-drag p-2 hover:bg-futou-border rounded cursor-pointer active:scale-90"
      >
        <Minus size={14} className="text-futou-text-secondary" />
      </button>
      <button
        onClick={() => appWindow.toggleMaximize()}
        className="no-drag p-2 hover:bg-futou-border rounded cursor-pointer active:scale-90"
      >
        {maximized ? (
          <Copy size={13} className="text-futou-text-secondary" />
        ) : (
          <Square size={13} className="text-futou-text-secondary" />
        )}
      </button>
      <button
        onClick={() => appWindow.close()}
        className="no-drag p-2 hover:bg-futou-danger hover:text-white rounded cursor-pointer active:scale-90"
      >
        <X size={15} />
      </button>
    </div>
  );
}
