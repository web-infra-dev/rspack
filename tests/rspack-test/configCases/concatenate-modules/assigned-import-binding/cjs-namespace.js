import * as cjsObject from "./cjs-object";

export function writeCjsExport() {
	cjsObject.value = 2;
	return cjsObject.value;
}
