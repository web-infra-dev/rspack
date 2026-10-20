import * as ns from "../mangle-escaping-namespace/enums";

export const getEnums = () => ns;
export function read() {
	const { ENUM_B, NUM } = ns;
	return [ENUM_B, NUM];
}
