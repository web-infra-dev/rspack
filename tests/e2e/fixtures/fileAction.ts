import path from "node:path";
import fs from "fs-extra";
import type { Fixtures } from "@playwright/test";
import type { RspackFixtures } from "./rspack";

type FileAction = {
	renameFile(oldPath: string, newPath: string): void;
	updateFile(relativePath: string, fn: (content: string) => string): void;
	deleteFile(relativePath: string): void;
	readDistFile(relativePath: string): string | void;
};

type FileActionFixtures = {
	fileAction: FileAction;
};

export const fileActionFixtures: Fixtures<
	FileActionFixtures,
	{},
	RspackFixtures
> = {
	fileAction: async ({ rspack }, use) => {
		// null means this file needs to be deleted
		const fileOriginContent: Record<string, string | null> = {};

		await use({
			renameFile(oldPath, newPath) {
				const oldFilePath = path.resolve(rspack.projectDir, oldPath);
				const newFilePath = path.resolve(rspack.projectDir, newPath);
				fs.renameSync(oldFilePath, newFilePath);
			},
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
			},
			readDistFile(relativePath) {
				const filePath = path.resolve(rspack.outDir, relativePath);
				const fileExists = fs.existsSync(filePath);
				if (!fileExists) {
					return;
				}
				return fs.readFileSync(filePath, "utf-8");
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
