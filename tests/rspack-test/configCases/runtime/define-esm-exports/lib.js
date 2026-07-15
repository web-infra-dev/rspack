export { readValue } from "./cycle.js";

export let value = 1;

export function setValue(next) {
  value = next;
}
