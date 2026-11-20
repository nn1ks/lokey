import type { Component } from "solid-js";

export const DefaultHeaderTitle: Component = () => {
  return (
    <div class="flex items-center gap-2 px-1">
      <span class="block size-5.5">
        <img
          src="/logo-black.svg"
          alt=""
          width="24"
          height="24"
          class="size-full -mt-px dark:hidden"
        />
        <img
          src="/logo-white.svg"
          alt=""
          width="24"
          height="24"
          class="hidden size-full -mt-px dark:block"
        />
      </span>
      <span class="select-none text-md font-semibold text-neutral-800 dark:text-neutral-200">
        Lokey Editor
      </span>
    </div>
  );
};

export default DefaultHeaderTitle;
