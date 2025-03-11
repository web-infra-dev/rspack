import { getResult } from "./wasm.wasm";

export const result = getResult(1);

export function getNumber() {
	return 20;
}
