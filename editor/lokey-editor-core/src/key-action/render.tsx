import type { Component, JSX, ParentComponent, ParentProps } from "solid-js";

export const Small: ParentComponent = (props) => (
  <div class="text-[0.75em]">{props.children}</div>
);

export const TextSubtitleCombination: Component<{
  text: string;
  subtitle: string;
}> = (props) => (
  <div class="relative mt-0.5 flex flex-col items-center">
    <div>{props.text}</div>
    <div class="text-[9px] leading-2.5">{props.subtitle}</div>
  </div>
);

export const SmallTextSubtitleCombination: Component<{
  text: string;
  subtitle: string;
}> = (props) => (
  <div class="relative mt-0.5 flex flex-col items-center">
    <div class="text-[0.75em]">{props.text}</div>
    <div class="text-[9px] leading-2.5">{props.subtitle}</div>
  </div>
);

export const IconSubtitleCombination: Component<{
  icon: JSX.Element;
  subtitle: string;
}> = (props) => (
  <div class="relative mt-0.5 flex flex-col items-center gap-1">
    <div>{props.icon}</div>
    <div class="-mt-1 text-[9px] leading-2.5">{props.subtitle}</div>
  </div>
);

export const Labelled: Component<ParentProps<{ label: string }>> = (props) => (
  <div class="relative flex h-full w-full items-center justify-center">
    <div
      class="
        absolute left-0 ml-0.5 h-full text-center text-[7px] font-semibold
        tracking-widest text-neutral-500 [text-orientation:sideways]
        [writing-mode:sideways-lr] uppercase
      "
    >
      {props.label}
    </div>
    <div class="px-3">{props.children}</div>
  </div>
);

export const Split: Component<{
  action1: JSX.Element;
  action2: JSX.Element;
}> = (props) => (
  <div class="h-full w-full">
    <div class="flex h-1/2 min-h-1/2 items-center justify-center text-[0.8em]">
      {props.action1}
    </div>
    <div
      class="
        absolute top-1/2 w-full -translate-y-1/2 border-t-[1.8px]
        border-dashed border-neutral-400 dark:border-neutral-600
      "
    />
    <div class="flex h-1/2 min-h-1/2 items-center justify-center text-[0.8em]">
      {props.action2}
    </div>
  </div>
);
