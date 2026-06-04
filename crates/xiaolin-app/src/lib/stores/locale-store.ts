import { create } from "zustand";
import { persist } from "zustand/middleware";
import i18n from "../../i18n";

export type Locale = "zh" | "en";
export type ResponseLang = "zh" | "en" | "follow-ui" | "auto";

export interface LocaleState {
  locale: Locale;
  responseLang: ResponseLang;
  setLocale: (locale: Locale) => void;
  setResponseLang: (responseLang: ResponseLang) => void;
  /** Resolve effective response language code for backend */
  resolvedResponseLang: () => string | null;
}

export const useLocaleStore = create<LocaleState>()(
  persist(
    (set, get) => ({
      locale: "zh" as Locale,
      responseLang: "zh" as ResponseLang,
      setLocale: (locale) => {
        i18n.changeLanguage(locale);
        set({ locale });
      },
      setResponseLang: (responseLang) => set({ responseLang }),
      resolvedResponseLang: () => {
        const { responseLang, locale } = get();
        switch (responseLang) {
          case "zh": return "zh";
          case "en": return "en";
          case "follow-ui": return locale;
          case "auto": return null;
        }
      },
    }),
    { name: "xiaolin-locale" },
  ),
);
