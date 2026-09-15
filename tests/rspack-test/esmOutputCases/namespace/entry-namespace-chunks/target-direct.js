import { directPrivate } from "./direct-private";
import { sharedValue, trace } from "./shared";

trace.push("direct");

export const foo = `${sharedValue}:${directPrivate}`;
export const bar = "DROP_BAR_SENTINEL";
export const collision = "target-collision";
export const directOnly = "direct-only";
export let counter = 0;
export function increment() {
  counter++;
}
