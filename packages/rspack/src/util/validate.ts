import type { z } from "zod";
import { fromZodError } from "zod-validation-error";

export class ValidationError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "ValidationError";
	}
}

export function validate<T extends z.ZodType>(opts: any, schema: T) {
	const res = schema.safeParse(opts);
	if (!res.success) {
		const strategy = process.env.RSPACK_CONFIG_VALIDATE ?? "strict";
		if (strategy === "loose-silent") return;

		let friendlyErr: ValidationError;
		const originalIssues = res.error.issues;

		// Issues with code `unrecognized_keys` are ignored. Other issues are thrown.
		// This does not work when `zodError.issues` is empty so we need to check the length of `zodError.issues`.
		// See: https://github.com/causaly/zod-validation-error/blob/62684ba47cba9cbd84e2a75dfd5fd06dcd0e1ad5/lib/fromZodError.ts#L53
		if (strategy === "loose-unrecognized-keys" && res.error.issues.length > 0) {
			// This is based on the invariant that `fromZodError` always reads `zodError.errors` first and
			// `zodError.errors` always returns `zodError.issues`:
			// See: https://github.com/causaly/zod-validation-error/blob/62684ba47cba9cbd84e2a75dfd5fd06dcd0e1ad5/lib/fromZodError.ts#L55
			// Also see: https://github.com/colinhacks/zod/blob/8552233c77426f77d3586cc877f7aec1aa0aa45b/src/ZodError.ts#L200
			const unrecognizedKeys = originalIssues.filter(
				issue => issue.code === "unrecognized_keys"
			);
			if (unrecognizedKeys.length > 0) {
				res.error.issues = unrecognizedKeys;
				friendlyErr = toValidationError(res.error);
				console.error(friendlyErr.message);
				res.error.issues = originalIssues;
			}
			res.error.issues = originalIssues.filter(
				issue => issue.code !== "unrecognized_keys"
			);
			if (res.error.issues.length > 0) {
				throw toValidationError(res.error);
			}
			return;
		}

		if (strategy === "loose-unrecognized-keys" || strategy === "loose") {
			friendlyErr = toValidationError(res.error);
			console.error(friendlyErr.message);
			return;
		}

		// strict
		friendlyErr = toValidationError(res.error);
		throw friendlyErr;
	}
}

function toValidationError(error: z.ZodError): ValidationError {
	const issueSeparator = "$issue$";
	const prefixSeparator = "$prefix$";
	const validationErr = fromZodError(error, {
		prefix:
			"Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.",
		prefixSeparator,
		issueSeparator
	});
	// The output validationErr.message looks like
	// `Configuration error$prefix$xxxx error$issue$yyy error$issue$zzz error`
	const [prefix, reason] = validationErr.message.split(prefixSeparator);
	const reasonItem = reason.split(issueSeparator);
	const message = `${prefix}\n${reasonItem.map(item => `- ${item}`).join("\n")}`;
	const friendlyErr = new ValidationError(message);
	return friendlyErr;
}

export function isValidate<T extends z.ZodType>(opts: any, schema: T) {
	try {
		validate(opts, schema);
		return true;
	} catch {
		return false;
	}
}
