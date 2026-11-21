import {
  createContext,
  createEffect,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import type { Accessor, ParentComponent } from "solid-js";

export type Theme = "light" | "dark";
export type ThemePreference = Theme | "system";

export type ThemeContextValue = {
  /** The theme currently applied to the document. */
  theme: Accessor<Theme>;
  /** The user's theme preference, which may follow the system theme. */
  themePreference: Accessor<ThemePreference>;
  setThemePreference: (preference: ThemePreference) => void;
};

export const ThemeContext = createContext<ThemeContextValue>();

const storageKey = "lokey-theme";

function storedThemePreference(): ThemePreference {
  if (typeof window === "undefined") {
    return "system";
  }

  const stored = window.localStorage.getItem(storageKey);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }

  return "system";
}

function systemTheme(): Theme {
  if (typeof window === "undefined") {
    return "light";
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export const ThemeProvider: ParentComponent = (props) => {
  const [themePreference, setThemePreference] = createSignal<ThemePreference>(
    storedThemePreference(),
  );
  const [system, setSystem] = createSignal<Theme>(systemTheme());

  if (typeof window !== "undefined") {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = (event: MediaQueryListEvent) => {
      setSystem(event.matches ? "dark" : "light");
    };

    media.addEventListener("change", handleChange);
    onCleanup(() => media.removeEventListener("change", handleChange));
  }

  const theme = (): Theme => {
    const preference = themePreference();
    return preference === "system" ? system() : preference;
  };

  createEffect(() => {
    const current = theme();
    const preference = themePreference();

    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", current === "dark");
      document.documentElement.style.colorScheme = current;
    }

    if (typeof window !== "undefined") {
      window.localStorage.setItem(storageKey, preference);
    }
  });

  const value: ThemeContextValue = {
    theme,
    themePreference,
    setThemePreference,
  };

  return (
    <ThemeContext.Provider value={value}>
      {props.children}
    </ThemeContext.Provider>
  );
};

export const useTheme = () => {
  const context = useContext(ThemeContext);

  if (!context) {
    throw new Error("useTheme must be used inside a ThemeProvider");
  }

  return context;
};
