import path from "path";
import fs from "fs-extra";
import { Fixtures } from "@playwright/test";
import type { RspackFixtures } from "./rspack";

type FileAction = {
	updateFile(relativePath: string, fn: (content: string) => string): void;
	deleteFile(relativePath: string): void;
};

type FileActionFixtures = {
	fileAction: FileAction;
};

export const fileActionFixtures: Fixtures<
	FileActionFixtures,
	{},
	RspackFixtures
> = {
	fileAction: async function ({ rspack }, use) {
		// null means this file needs to be deleted
		const fileOriginContent: Record<string, string | null> = {};

		await use({
			updateFile(relativePath, fn) {
				const filePath = path.resolve(rspack.projectDir, relativePath);
				const fileExists = fs.existsSync(filePath);
				const content = fileExists ? fs.readFileSync(filePath).toString() : "";

				if (fileOriginContent[filePath] === undefined) {
					fileOriginContent[filePath] = fileExists ? content : null;
				}

				fs.writeFileSync(filePath, fn(content));
			},
			deleteFile(relativePath) {
				const filePath = path.resolve(rspack.projectDir, relativePath);
				const fileExists = fs.existsSync(filePath);
				if (!fileExists) {
					return;
				}

				if (fileOriginContent[filePath] === undefined) {
					fileOriginContent[filePath] = fs.readFileSync(filePath).toString();
				}

				fs.unlinkSync(filePath);
			}
		});

		for (const [filePath, content] of Object.entries(fileOriginContent)) {
			if (content === null) {
				fs.unlinkSync(filePath);
			} else {
				fs.writeFileSync(filePath, content);
			}
		}
		if (Object.keys(fileOriginContent).length) {
			// has recovery file
			await rspack.waitingForBuild();
		}
	}
};
