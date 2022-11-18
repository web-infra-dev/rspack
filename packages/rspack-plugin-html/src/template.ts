import fs from "node:fs/promises";
import path from "node:path";

/**
 * compile template to js code.
 */
export async function compile(templatePath: string): Promise<string> {
	return fs.readFile(templatePath, "utf-8");
}

/**
 * eval js code to js function.
 */
export async function evaluate(compiled: string): Promise<string> {
	return compiled;
}
