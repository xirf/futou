import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface CatalogueEntry {
  runtime: string;
  versions: string[];
}

export function useCatalogue() {
  const [catalogue, setCatalogue] = useState<CatalogueEntry[]>([]);

  const fetchCatalogue = useCallback(async () => {
    try {
      const res = await invoke<string>("catalogue_list");
      const data = JSON.parse(res);
      if (data.result?.runtimes) {
        setCatalogue(data.result.runtimes.map((r: { name: string; versions: string[] }) => ({
          runtime: r.name,
          versions: r.versions,
        })));
      }
    } catch { /* daemon offline */ }
  }, []);

  return { catalogue, fetchCatalogue };
}
