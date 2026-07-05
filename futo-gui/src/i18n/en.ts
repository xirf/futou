const en: Record<string, string> = {
  "app.title": "Futou",
  "app.subtitle": "{count} service, {activeCount} active",
  "app.daemonOn": "ON",
  "app.daemonOff": "OFF",
  "app.add": "Add",
  "app.settings": "Settings",
  "app.back": "Back",
  "app.save": "Save",
  "app.saved": "Saved",

  "settings.title": "Settings",
  "settings.language": "Language",
  "settings.languageEn": "English",
  "settings.languageId": "Indonesian",
  "settings.installLocation": "Install Location",
  "settings.installLocationDesc":
    "Directory where runtimes (PHP, Node.js, MariaDB, etc.) are stored.",
  "settings.browse": "Browse",
  "settings.daemonAutostart": "Run daemon at startup",
  "settings.daemonAutostartDesc":
    "Automatically start the futou daemon when you log in to Windows.",

  "service.start": "Start",
  "service.stop": "Stop",
  "service.restart": "Restart",
  "service.config": "Config",
  "service.openDir": "Open directory",
  "service.uninstall": "Uninstall",
  "service.switching": "Switching version...",
  "service.noRuntimes": "No runtimes installed yet.",

  "modal.title": "Add Service",
  "modal.searchPlaceholder": "Search services...",
  "modal.installing": "Installing",
  "modal.install": "Install",
  "modal.close": "Close",
  "modal.noCatalogue": "Catalogue is empty or unavailable.",
  "modal.noResults": "No matching services found.",

  "logs.title": "Logs",
  "logs.empty": "Select a service to view logs.",

  "daemon.failedStart": "Daemon failed to start",
  "daemon.starting": "Starting daemon...",

  "error.activate": "Failed to activate {runtime}: {error}",
  "error.deactivate": "Failed to deactivate {runtime}: {error}",
  "error.start": "Failed to start {runtime}: {error}",
  "error.stop": "Failed to stop {runtime}: {error}",
  "error.switch": "Failed to switch {runtime}: {error}",
  "error.open": "Failed to open: {error}",
  "error.generic": "Error: {error}",

  "confirm.switchDb": "Switching {runtime} from {from} to {to}.\n\nThe server will be stopped.\n{runtimeUpper} is a database — different versions may cause data incompatibility.\nContinue?",
  "confirm.switch": "Switching {runtime} from {from} to {to}.\n\nThe server will be stopped and reactivated with the new version.\nContinue?",
  "confirm.uninstall": "Uninstall {runtime} {version}?\n\nThis will delete all files and cannot be undone.",
};

export default en;
