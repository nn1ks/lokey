import type { ParentComponent } from "solid-js";

export const StartPage: ParentComponent = (props) => {
  return (
    <main class="flex h-[calc(100vh-var(--header-height))] items-center justify-center">
      <div class="flex w-full max-w-76 flex-col items-center gap-4">
        {props.children}
      </div>
    </main>
  );
};

export default StartPage;
