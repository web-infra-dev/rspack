// ESM module with named exports for tree-shaking macro testing
export function usedUtil() {
	return "used utility function";
}

export function unusedUtil() {
	return "unused utility function";
}

export function processEsmData(data) {
	return "ESM processed: " + data;
}

export function validateData(data) {
	return typeof data === "string" && data.length > 0;
}

export function unusedEsmFunction() {
	return "unused ESM function";
}

export const ESM_CONSTANT = "ESM constant";
export const UNUSED_ESM_CONSTANT = "unused ESM constant";

// Default export that should also be tree-shaken
export default function defaultFunction() {
	return "default function";
}