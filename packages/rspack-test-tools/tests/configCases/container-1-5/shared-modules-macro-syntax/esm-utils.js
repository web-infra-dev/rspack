// ESM module for testing export specifier syntax
export function usedUtil() {
	return "This utility is used";
}

export function unusedUtil() {
	return "unused utility function";
}

export const USED_CONSTANT = "used constant";
export const UNUSED_CONSTANT = "unused constant";

export default function defaultExport() {
	return "default export function";
}
