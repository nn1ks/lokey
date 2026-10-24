import type { Component } from "solid-js";

export const PoweredByLokey: Component = () => {
  return (
    <div class="mt-12 w-full text-center italic text-neutral-400 dark:text-neutral-600">
      Powered by{" "}
      <a
        class="
          text-inherit underline decoration-dashed
          hover:text-neutral-600 dark:hover:text-neutral-400
        "
        href="https://lokey.rs"
        target="_blank"
        rel="noopener noreferrer"
      >
        Lokey
      </a>
    </div>
  );
};

export default PoweredByLokey;
