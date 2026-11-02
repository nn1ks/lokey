import { Show } from "solid-js";
import type { Component, JSX } from "solid-js";

import { useConnection } from "../contexts";
import Button from "./Button";
import ThemeButton from "./ThemeButton";

type HeaderProps = {
  title: JSX.Element;
};

export const Header: Component<HeaderProps> = (props) => {
  const { connection, disconnect } = useConnection();

  return (
    <>
      <header
        class="
          fixed top-0 left-0 z-100 flex items-center justify-between
          h-(--header-height) px-4 w-full border-b
          border-neutral-300 bg-neutral-200/70
          dark:border-neutral-750 dark:bg-neutral-800/70
          backdrop-blur-md
        "
      >
        <div class="flex items-center gap-2">{props.title}</div>

        <div class="flex items-center gap-2">
          <Show
            when={
              connection().status === "connected" ? connection() : undefined
            }
          >
            {(connected) => {
              const isDemo = () => {
                const state = connected();
                return state.status === "connected" && state.isDemo;
              };

              return (
                <Button
                  class="
                    bg-neutral-400/30! hover:bg-neutral-500/30!
                    dark:bg-neutral-700/30! dark:hover:bg-neutral-600/30!
                  "
                  onClick={() => void disconnect()}
                >
                  {isDemo() ? "Leave Demo" : "Disconnect"}
                </Button>
              );
            }}
          </Show>
          <ThemeButton
            class="
              bg-neutral-400/30! hover:bg-neutral-500/30!
              dark:bg-neutral-700/30! dark:hover:bg-neutral-600/30!
            "
          />
        </div>
      </header>
      <div class="h-(--header-height)"></div>
    </>
  );
};

export default Header;
