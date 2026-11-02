import { Moon, Sun } from "lucide-solid";
import { Show, splitProps } from "solid-js";
import type { Component } from "solid-js";
import { useTheme } from "../contexts";
import type { ButtonProps } from "./Button";

export type ThemeButtonProps = Omit<
  ButtonProps,
  "type" | "onClick" | "title" | "aria-label" | "children"
>;

export const ThemeButton: Component<ThemeButtonProps> = (props) => {
  const { theme, toggleTheme } = useTheme();
  const [local, rest] = splitProps(props, ["class"]);

  const label = () =>
    theme() === "dark" ? "Switch to light mode" : "Switch to dark mode";

  return (
    <button
      {...rest}
      type="button"
      class={[
        "inline-flex size-9 items-center justify-center rounded-lg cursor-pointer",
        "bg-neutral-200 text-neutral-800 dark:bg-neutral-900 dark:text-neutral-200",
        "hover:bg-neutral-300 dark:hover:bg-neutral-800",
        "transition-[background-color] duration-50 ease-[ease]",
        "focus-visible:outline-2 focus-visible:outline-focused focus-visible:outline-offset-2",
        "[&_svg]:size-5",
        local.class ?? "",
      ]
        .filter(Boolean)
        .join(" ")}
      aria-label={label()}
      title={label()}
      onClick={() => toggleTheme()}
    >
      <Show when={theme() === "dark"} fallback={<Moon />}>
        <Sun />
      </Show>
    </button>
  );
};

export default ThemeButton;
