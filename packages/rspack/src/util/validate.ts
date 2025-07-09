import { createErrorMap, fromZodError } from "zod-validation-error/v4";
import type { z } from "../config/zod";

export class ValidationError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "ValidationError";
	}
}

export function validate<T extends z.ZodType>(
	opts: any,
	createSchema: () => T,
	options: {
		output?: boolean;
		strategy?: "strict" | "loose-unrecognized-keys" | "loose-silent" | "loose";
	} = {}
): string | null {
	const strategy =
		options.strategy ?? process.env.RSPACK_CONFIG_VALIDATE ?? "strict";

	// Skip schema validation if the strategy is `loose-silent`
	if (strategy === "loose-silent") {
		return null;
	}

	const schema =
		typeof createSchema === "function" ? createSchema() : createSchema;
	const res = schema.safeParse(opts);

	if (!res.success) {
		const output = options.output ?? true;

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
				friendlyErr = toValidationError({
					...res.error,
					issues: unrecognizedKeys
				});
				if (output) {
					console.error(friendlyErr.message);
				}
			}
			const issuesWithoutUnrecognizedKeys = originalIssues.filter(
				issue => issue.code !== "unrecognized_keys"
			);
			if (issuesWithoutUnrecognizedKeys.length > 0) {
				throw toValidationError({
					...res.error,
					issues: issuesWithoutUnrecognizedKeys
				});
			}
			return output || !friendlyErr! ? null : friendlyErr.message;
		}

		if (strategy === "loose-unrecognized-keys" || strategy === "loose") {
			friendlyErr = toValidationError(res.error);
			if (output) {
				console.error(friendlyErr.message);
			}
			return output ? null : friendlyErr.message;
		}

		// strict
		friendlyErr = toValidationError(res.error);
		throw friendlyErr;
	}
	return null;
}

function toValidationError(error: z.ZodError): ValidationError {
	// Instead of using `z.config({ customError: createErrorMap() })` to customize the error message,
	// we use `createErrorMap` to customize the error message.
	// This gives us fine-grained control over the error messages.
	const customErrorMap = createErrorMap();
	const separator = "\n- ";
	const validationErr = fromZodError(error, {
		prefix:
			"Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.",
		prefixSeparator: separator,
		issueSeparator: separator,
		error: customErrorMap
	});
	return new ValidationError(validationErr.message);
}

export function isValidate<T extends z.ZodType>(
	opts: any,
	createSchema: () => T
) {
	try {
		validate(opts, createSchema);
		return true;
	} catch {
		return false;
	}
}
