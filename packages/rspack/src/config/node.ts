import { RawNodeOption } from "@rspack/binding";

export interface NodeOptions {
	/**
	 * Include a polyfill for the '__dirname' variable.
	 */
	__dirname?: boolean | "warn-mock" | "mock" | "eval-only";
}

export function resolveNode(node?: NodeOptions): RawNodeOption {
	if (node == null) {
		return {};
	}
	return {
		dirname:
			typeof node.__dirname === "boolean"
				? String(node.__dirname)
				: node.__dirname
	};
}
