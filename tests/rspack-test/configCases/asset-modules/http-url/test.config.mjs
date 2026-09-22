import fs from "node:fs";
import path from "node:path";

export default {
	beforeExecute() {
		try {
			fs.unlinkSync(path.join(import.meta.dirname, "dev-defaults.webpack.lock"));
		} catch (e) {}
	},
	afterExecute() {
		try {
			fs.unlinkSync(path.join(import.meta.dirname, "dev-defaults.webpack.lock"));
		} catch (e) {}
	}
};
