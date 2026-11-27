(globalThis.order ??= []).push("a");
export let value = "a";
export function set(next) { value = next; }
export default "a";
