import { getResult } from "./wasm.wasm";
console.log(getResult);
export var result = getResult(1);

export function getNumber() {
	return 20;
}
