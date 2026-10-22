import fs from "node:fs";
import path from "node:path";
export default function() {
	return {
		code: fs.readFileSync(path.join(path.dirname(import.meta.filename), "stylesheet.css"), "utf-8") + "\n.generated { color: red; }"
	};
};
