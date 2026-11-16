import fs from "node:fs";
import path from "node:path";

export default {
	moduleScope(scope) {
		scope.__nodeFs = fs;
		scope.__nodePath = path;
		scope.__NodeBuffer = Buffer;
	}
};
