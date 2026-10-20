import * as ns from "./enums";

export const getEnums = () => ns;
export const getAgain = () => ns;
export const spread = () => ({ ...ns });
export const missing = () => ns.missing;
export const deleteMissing = () => [delete ns.missing, delete ns.anotherMissing];
export function read() {
	const { ENUM_B, NUM: value = 0 } = ns;
	return [ENUM_B, value];
}
