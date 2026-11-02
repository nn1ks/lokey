import type { Component } from "solid-js";

export const DefaultHeaderTitle: Component = () => {
  return (
    <>
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
      <span class="select-none text-md font-semibold">Lokey Editor</span>
    </>
  );
};

export default DefaultHeaderTitle;
