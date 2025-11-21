export function usedLocalUtil() {
	return "This local utility is used";
}

export function unusedLocalUtil() {
	return "unused local utility function";
}

export const USED_LOCAL_CONSTANT = "used local constant";
export const UNUSED_LOCAL_CONSTANT = "unused local constant";

export const utilityHelpers = {
	helper1: () => "helper1",
	helper2: () => "helper2"
};

export default function localDefaultExport() {
	return "local default export function";
}
