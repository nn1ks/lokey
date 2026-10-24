import type { Component } from "solid-js";

export const DefaultHeaderTitle: Component = () => {
  return (
    <>
      <picture class="size-5.5 opacity-90">
        <source media="(prefers-color-scheme: dark)" srcset="/logo-white.svg" />
        <img src="/logo-black.svg" alt="" width="24" height="24" />
      </picture>
      <span class="select-none text-[1.075rem] font-semibold">
        Lokey Editor
      </span>
    </>
  );
};

export default DefaultHeaderTitle;
