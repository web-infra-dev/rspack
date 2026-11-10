export let x = 1;
export function inc() {
	x++;
}
export const z = 3;

// These names have opposite relative order in UTF-16 and UTF-8.
export { x as "\u{10000}", inc as "\uE000" };
