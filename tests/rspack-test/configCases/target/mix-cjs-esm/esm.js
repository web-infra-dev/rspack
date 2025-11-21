import cjs from "./lib/cjs";
import { getFilePath as esmGetFilePath } from "./lib/esm";

export async function getFilePath() {
	const cjsLib = await cjs.getFilePath();
	const esmLib = await esmGetFilePath();
	return ["esm.js", cjsLib, esmLib].join("|");
}
