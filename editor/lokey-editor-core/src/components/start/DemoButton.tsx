import type { Component } from "solid-js";
import { Play } from "lucide-solid";
import StartButton from "./StartButton";
import type { StartButtonProps } from "./StartButton";
import { useConnection } from "../../contexts";

export type DemoButtonProps = Omit<StartButtonProps, "icon" | "label">;

export const DemoButton: Component<DemoButtonProps> = (props) => {
  const { connectDemo } = useConnection();

  return (
    <StartButton
      {...props}
      icon={Play}
      label="Demo Mode"
      onClick={connectDemo}
    />
  );
};

export default DemoButton;
