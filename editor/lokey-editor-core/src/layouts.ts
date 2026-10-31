/** Host layouts supported by the editor, in display order. */
export const layouts = [
  { value: "english-us", label: "English (US)" },
  { value: "german", label: "German" },
] as const;

export type Layout = (typeof layouts)[number]["value"];

export function isLayout(value: string): value is Layout {
  return layouts.some((layout) => layout.value === value);
}
