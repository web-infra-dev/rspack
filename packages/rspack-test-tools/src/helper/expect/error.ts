import prettyFormat from "pretty-format";

const ERROR_STACK_PATTERN = /(│.* at ).*/g;

const prettyFormatOptions = {
	escapeRegex: false,
	printFunctionName: false,
	plugins: [
		{
			test(val: any) {
				return typeof val === "string";
			},
			print(val: any) {
				return `"${val
					.replace(/\\/gm, "/")
					.replace(/"/gm, '\\"')
					.replace(/\r?\n/gm, "")}"`;
			}
		}
	]
};

function cleanErrorStack(message: string) {
	return message.replace(ERROR_STACK_PATTERN, "$1xxx");
}

function cleanError(err: Error) {
	const result: Partial<Record<keyof Error, any>> = {};
	for (const key of Object.getOwnPropertyNames(err)) {
		result[key as keyof Error] = err[key as keyof Error];
	}

	if (result.message) {
		result.message = cleanErrorStack(err.message);
	}

	if (result.stack) {
		result.stack = cleanErrorStack(result.stack);
	}

	return result;
}

export function normalizeDignostics(received: {
	errors: Error[];
	warnings: Error[];
}): string {
	return prettyFormat(
		{
			errors: received.errors.map(e => cleanError(e)),
			warnings: received.warnings.map(e => cleanError(e))
		},
		prettyFormatOptions
	).trim();
}

export function normalizeError(received: Error): string {
	return prettyFormat(cleanError(received), prettyFormatOptions).trim();
}
