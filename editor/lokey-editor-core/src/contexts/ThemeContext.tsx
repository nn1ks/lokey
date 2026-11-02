import { createContext, createEffect, createSignal, useContext } from "solid-js";
import type { Accessor, ParentComponent } from "solid-js";

export type Theme = "light" | "dark";

export type ThemeContextValue = {
  theme: Accessor<Theme>;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
};

export const ThemeContext = createContext<ThemeContextValue>();

const storageKey = "lokey-theme";

function preferredTheme(): Theme {
  if (typeof window === "undefined") {
    return "light";
  }

  const stored = window.localStorage.getItem(storageKey);
  if (stored === "light" || stored === "dark") {
    return stored;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export const ThemeProvider: ParentComponent = (props) => {
  const [theme, setTheme] = createSignal<Theme>(preferredTheme());

  createEffect(() => {
    const current = theme();

    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", current === "dark");
      document.documentElement.style.colorScheme = current;
    }

    if (typeof window !== "undefined") {
      window.localStorage.setItem(storageKey, current);
    }
  });

  const value: ThemeContextValue = {
    theme,
    setTheme,
    toggleTheme: () =>
      setTheme((current) => (current === "dark" ? "light" : "dark")),
  };

  return (
    <ThemeContext.Provider value={value}>{props.children}</ThemeContext.Provider>
  );
};

export const useTheme = () => {
  const context = useContext(ThemeContext);

  if (!context) {
    throw new Error("useTheme must be used inside a ThemeProvider");
  }

  return context;
};
