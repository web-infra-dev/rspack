import { cleanUp } from "../ErrorHelpers";
import WebpackError from "../lib/WebpackError";

export default class ModuleError extends WebpackError {
	error?: Error;

	constructor(err: Error, { from }: { from?: string } = {}) {
		let message = "Module Error";

		if (from) {
			message += ` (from ${from}):\n`;
		} else {
			message += ": ";
		}

		if (err && typeof err === "object" && err.message) {
			message += err.message;
		} else if (err) {
			message += err;
		}

		super(message);

		this.name = "ModuleError";
		this.error = err;
		this.details =
			err && typeof err === "object" && err.stack
				? cleanUp(err.stack, err.name, err.message)
				: undefined;
	}
}
