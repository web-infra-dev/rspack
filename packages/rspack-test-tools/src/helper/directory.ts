import fs from "fs";
import path from "path";

import { escapeSep } from ".";

export const isDirectory = (p: string) => fs.lstatSync(p).isDirectory();
export const isFile = (p: string) => fs.lstatSync(p).isFile();
export const isValidCaseDirectory = (name: string) =>
	!name.startsWith("_") && !name.startsWith(".") && name !== "node_modules";

export function describeByWalk(
	testFile: string,
	createCase: (name: string, src: string, dist: string) => void,
	options: {
		type?: "file" | "directory";
		level?: number;
		source?: string;
		dist?: string;
		absoluteDist?: boolean;
	} = {}
) {
	const testBasename = path
		.basename(testFile)
		.replace(/\.(diff|hot)?test\.js/, "");
	const testId = testBasename.charAt(0).toLowerCase() + testBasename.slice(1);
	const sourceBase =
		options.source || path.join(path.dirname(testFile), `${testId}Cases`);
	const distBase =
		options.dist || path.join(path.dirname(testFile), "js", testId);
	const level = options.level || 2;
	const type = options.type || "directory";
	const absoluteDist = options.absoluteDist ?? true;
	function describeDirectory(dirname: string, currentLevel: number) {
		fs.readdirSync(path.join(sourceBase, dirname))
			.filter(isValidCaseDirectory)
			.filter(folder => {
				const p = path.join(sourceBase, dirname, folder);
				if (type === "file" && currentLevel === 1) {
					return isFile(p);
				} else if (type === "directory" || currentLevel > 1) {
					return isDirectory(p);
				} else {
					return false;
				}
			})
			.map(folder => {
				const caseName = path.join(dirname, folder);
				if (currentLevel > 1) {
					describeDirectory(caseName, currentLevel - 1);
				} else {
					const name = escapeSep(
						path.join(testId, caseName).split(".").shift()!
					);
					describe(name, () => {
						let source = path.join(sourceBase, caseName);
						let dist = "";
						if (absoluteDist) {
							dist = path.join(distBase, caseName);
						} else {
							const relativeDist = options.dist || "dist";
							if (path.isAbsolute(relativeDist)) {
								dist = path.join(relativeDist, caseName);
							} else {
								dist = path.join(sourceBase, caseName, relativeDist);
							}
						}
						createCase(name, source, dist);
					});
				}
			});
	}

	describe(testId, () => {
		describeDirectory("", level);
	});
}
