import type { Component, ParentProps } from "solid-js";

export const WithoutPagePadding: Component<ParentProps> = (props) => {
  return (
    <div class="w-[calc(100%+2*var(--page-padding-x))] -ml-(--page-padding-x)">
      {props.children}
    </div>
  );
};

export default WithoutPagePadding;
