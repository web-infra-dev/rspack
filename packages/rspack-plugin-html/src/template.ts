import fs from "node:fs/promises";
import path from "node:path";

/**
 * compile template to js code.
 */
export interface TemplateCompiler<T = any> {
	compile(content: string, options?: T): Promise<string>;
	options?: T;
}

export const defaultTemplateCompiler: TemplateCompiler = {
	async compile(content) {
		const template = (await import("lodash.template")).default(content, {
			interpolate: /<%=([\s\S]+?)%>/g,
			variable: "data"
		});
		return template.source;
	}
};

/**
 * eval js code to js function or js string.
 */
export async function evaluate(
	compiled: string,
	publicPath: string,
	templateFilename: string
): Promise<string | (() => string | Promise<string>)> {
	if (!compiled) {
		return Promise.reject(
			new Error("The templateCompiler didn't provide a result")
		);
	}
}
