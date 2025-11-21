// Utility module that consumes shared dependencies
import { isEmpty, isArray, isObject } from "lodash-es";

export function validateData(data) {
	if (isEmpty(data)) {
		return { valid: false, reason: "Data is empty" };
	}

	if (isArray(data)) {
		return { valid: true, type: "array", length: data.length };
	}

	if (isObject(data)) {
		return { valid: true, type: "object", keys: Object.keys(data).length };
	}

	return { valid: true, type: typeof data };
}

// Note: isArray and isObject are used, isEmpty is also used
// This should be reflected in ShareUsagePlugin output
