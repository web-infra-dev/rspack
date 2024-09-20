import {
	BuiltinPluginName,
	type RawContextReplacementPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export const ContextReplacementPlugin = create(
	BuiltinPluginName.ContextReplacementPlugin,
	(
		resourceRegExp: RegExp,
		newContentResource?: any,
		newContentRecursive?: any,
		newContentRegExp?: any
	) => {
		const rawOptions: RawContextReplacementPluginOptions = {
			resourceRegExp
		};
		if (typeof newContentResource === "function") {
			// rawOptions.newContentCallback = newContentResource;
		} else if (
			typeof newContentResource === "string" &&
			typeof newContentRecursive === "object"
		) {
			rawOptions.newContentResource = newContentResource;
			rawOptions.newContentCreateContextMap = newContentRecursive;
		} else if (
			typeof newContentResource === "string" &&
			typeof newContentRecursive === "function"
		) {
			rawOptions.newContentResource = newContentResource;
			// rawOptions.newContentCreateContextMap = newContentRecursive;
		} else {
			if (typeof newContentResource !== "string") {
				// biome-ignore lint/style/noParameterAssign: based on webpack's logic
				newContentRegExp = newContentRecursive;
				// biome-ignore lint/style/noParameterAssign: based on webpack's logic
				newContentRecursive = newContentResource;
				// biome-ignore lint/style/noParameterAssign: based on webpack's logic
				newContentResource = undefined;
			}
			if (typeof newContentRecursive !== "boolean") {
				// biome-ignore lint/style/noParameterAssign: based on webpack's logic
				newContentRegExp = newContentRecursive;
				// biome-ignore lint/style/noParameterAssign: based on webpack's logic
				newContentRecursive = undefined;
			}
			rawOptions.newContentResource = newContentResource;
			rawOptions.newContentRecursive = newContentRecursive;
			rawOptions.newContentRegExp = newContentRegExp;
		}
		return rawOptions;
	}
);
