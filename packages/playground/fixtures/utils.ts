import fs from "fs";
import path from "path";

const { testPath } = expect.getState();
if (!testPath) {
	throw Error("unreachable");
}
const caseName = testPath
	.replace(/\\/g, "/")
	.match(/playground\/fixtures\/([\w-]+)\//)?.[1];
export const testDir = path.resolve(__dirname, "../temp", caseName!);

export function editFile(filename: string, replacer: (str: string) => string) {
	const resolved = path.resolve(testDir, filename);
	const content = fs.readFileSync(resolved, "utf-8");
	const modified = replacer(content);
	fs.writeFileSync(resolved, modified);
}

// TODO: maybe we need a options to distinguish `wait for hot` or `wait for liveLoad`?
export async function waitingUpdate(
	poll: () => string | Promise<string | undefined | null>,
	expected: string
): Promise<void> {
	const maxTries = 100;
	for (let tries = 0; tries < maxTries; tries++) {
		const actual = await poll() ?? "";
		if (actual.indexOf(expected) > -1 || tries === maxTries - 1) {
			expect(actual).toMatch(expected);
			break;
		} else {
			await wait(50);
		}
	}
}

async function wait(time: number) {
	return new Promise(resolve => {
		setTimeout(() => {
			resolve(undefined);
		}, time);
	});
}

export async function getComputedStyle(
	selector: string
): Promise<CSSStyleDeclaration> {
	await page.waitForSelector(selector);
	return await page.$eval(selector, ele =>
		JSON.parse(JSON.stringify(window.getComputedStyle(ele)))
	);
}
