import fs from "node:fs";
import path from "node:path";

export default function compilerNameLoader(source) {
	const { counterFile } = this.getOptions();
	const compilerName = this._compiler.name;
	fs.mkdirSync(path.dirname(counterFile), { recursive: true });
	fs.appendFileSync(counterFile, compilerName);
	return source.replace("__COMPILER_NAME__", `compiler:${compilerName}`);
}
