import fs from "fs";
import { globSync } from "glob";
import path from "path";

export const isDirectory = (p: string) => fs.lstatSync(p).isDirectory();
export const isValidCaseDirectory = (name: string) =>
	!name.startsWith("_") && !name.startsWith(".");

export function describeByWalk(
	name: string,
	sourceBase: string,
	distBase: string,
	createCase: (name: string, src: string, dist: string) => void,
	whitelist: {
		cat?: RegExp;
		case?: RegExp;
	} = {}
) {
	const categories = fs
		.readdirSync(sourceBase)
		.filter(isValidCaseDirectory)
		.filter(folder => isDirectory(path.join(sourceBase, folder)))
		.filter(i => (whitelist?.cat ? whitelist.cat.test(i) : true))
		.map(cat => {
			return {
				name: cat,
				tests: fs
					.readdirSync(path.join(sourceBase, cat))
					.filter(isValidCaseDirectory)
					.filter(folder => isDirectory(path.join(sourceBase, cat, folder)))
					.filter(i => (whitelist?.case ? whitelist.case.test(i) : true))
					.sort()
			};
		});
	describe(name, () => {
		for (let { name: catName, tests } of categories) {
			if (tests.length === 0) continue;
			describe(catName, () => {
				for (const testName of tests) {
					const src = path.join(sourceBase, catName, testName);
					const dist = path.join(distBase, catName, testName);
					createCase(testName, src, dist);
				}
			});
		}
	});
}
