// ESM module with named exports for tree-shaking macro testing
import { pureFunction } from "./pure-helper.js";

export function usedUtil() {
	return "This utility is used";
}

export function unusedUtil() {
	return "unused utility function";
}

export function processEsmData(data) {
	// Use the imported function to ensure the import is included
	const factor = pureFunction(1);
	return data.value * factor;
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