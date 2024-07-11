import type { z } from "zod";
import { fromZodError } from "zod-validation-error";

export function validate<T extends z.ZodType>(opts: any, schema: T) {
	const res = schema.safeParse(opts);
	if (!res.success) {
		const strategy = process.env.RSPACK_CONFIG_VALIDATE ?? "strict";
		if (strategy === "loose-silent") return;
		const issueSeparator = "$issue$";
		const prefixSeparator = "$prefix$";
		const validationErr = fromZodError(res.error, {
			prefix: "Configuration error",
			prefixSeparator,
			issueSeparator
		});
		// The output validationErr.message looks like
		// `Configuration error$prefix$xxxx error$issue$yyy error$issue$zzz error`
		const [prefix, reason] = validationErr.message.split(prefixSeparator);
		const reasonItem = reason.split(issueSeparator);
		const friendlyErr = new Error(
			`${prefix}:\n${reasonItem.map(item => `- ${item}`).join("\n")}`
		);
		if (strategy === "loose") {
			console.error(friendlyErr.message);
		} else {
			throw friendlyErr;
		}
	}
}

export function isValidate<T extends z.ZodType>(opts: any, schema: T) {
	try {
		validate(opts, schema);
		return true;
	} catch {
		return false;
	}
}
