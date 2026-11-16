import fs from "node:fs";
import path from "node:path";

export default function compilerNameLoader(source) {
	const { counterFile } = this.getOptions();
	const compilerName = path.basename(this._compiler.outputPath);
	fs.mkdirSync(path.dirname(counterFile), { recursive: true });
	fs.appendFileSync(counterFile, compilerName);
	return source.replace("__COMPILER_NAME__", `compiler:${compilerName}`);
}
