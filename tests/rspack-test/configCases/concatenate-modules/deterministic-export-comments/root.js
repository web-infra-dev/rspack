import { marker } from "./dep";

export const unusedA = "unused-a";
export const unusedB = "unused-b";

export function usedA() {
	return `${marker()}-a`;
}

export function usedB() {
	return `${marker()}-b`;
}
