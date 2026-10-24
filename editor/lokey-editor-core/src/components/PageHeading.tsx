import type { Component, JSX } from "solid-js";

export type PageHeadingProps = {
  icon: JSX.Element;
  title: JSX.Element;
  description: JSX.Element;
  actions: JSX.Element;
};

export const PageHeading: Component<PageHeadingProps> = (props) => {
  return (
    <div class="box-border flex w-full items-center gap-3 py-4">
      <div
        class="
          flex size-11 flex-none items-center justify-center rounded-lg
          border border-neutral-200 bg-neutral-100
          dark:border-neutral-800 dark:bg-neutral-900
        "
      >
        {props.icon}
      </div>
      <div class="flex min-w-0 flex-1 flex-col">
        <h1 class="m-0 text-lg leading-6 font-medium text-neutral-800 dark:text-neutral-200">
          {props.title}
        </h1>
        <div class="text-md leading-5 text-neutral-500">
          {props.description}
        </div>
      </div>
      <div class="ml-auto flex flex-none items-center gap-2">
        {props.actions}
      </div>
    </div>
  );
};

export default PageHeading;
