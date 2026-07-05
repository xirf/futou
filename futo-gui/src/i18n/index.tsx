import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import en from "./en";
import id from "./id";

type Lang = "en" | "id";

const langs: Record<Lang, Record<string, string>> = { en, id };

interface I18nContextType {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: string, vars?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextType>({
  lang: "en",
  setLang: () => {},
  t: (k) => k,
});

function loadLang(): Lang {
  try {
    const stored = localStorage.getItem("futou_lang");
    if (stored === "id" || stored === "en") return stored;
  } catch {
    /* localStorage may be unavailable */
  }
  return "en";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(loadLang);

  const setLang = useCallback((l: Lang) => {
    setLangState(l);
    try {
      localStorage.setItem("futou_lang", l);
    } catch {
      /* ignore */
    }
  }, []);

  const t = useCallback(
    (key: string, vars?: Record<string, string | number>): string => {
      let text = langs[lang][key] ?? langs["en"][key] ?? key;
      if (vars) {
        for (const [k, v] of Object.entries(vars)) {
          text = text.replace(`{${k}}`, String(v));
        }
      }
      return text;
    },
    [lang],
  );

  return (
    <I18nContext.Provider value={{ lang, setLang, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useTranslation() {
  return useContext(I18nContext);
}
