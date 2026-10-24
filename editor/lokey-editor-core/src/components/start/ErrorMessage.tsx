import { CircleAlert } from "lucide-solid";
import type { ParentComponent } from "solid-js";

export const ErrorMessage: ParentComponent = (props) => {
  return (
    <div
      class="flex w-full items-center gap-3 p-0 text-red-700 dark:text-red-400 font-medium"
      role="alert"
    >
      <div class="flex size-[22px] items-center justify-center py-3">
        <CircleAlert size="22" />
      </div>
      <span>{props.children}</span>
    </div>
  );
};

export default ErrorMessage;
