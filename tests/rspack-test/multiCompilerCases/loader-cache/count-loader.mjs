import fs from "node:fs";
import path from "node:path";

export default function countLoader(source) {
	const { counterFile } = this.getOptions();
	fs.mkdirSync(path.dirname(counterFile), { recursive: true });
	fs.appendFileSync(counterFile, "x");
	return source;
}
