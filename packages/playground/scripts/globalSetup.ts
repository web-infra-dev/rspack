import fs from "fs-extra";
import os from "os";
import path from "path";
import { chromium } from "playwright-chromium";

const DIR = path.join(os.tmpdir(), "jest_playwright_ws_endpoint");

export default async function () {
	const browserServer = await chromium.launchServer({
		args: process.env.CI
			? ["--no-sandbox", "--disable-setuid-sandbox"]
			: undefined
	});

	global.browserServer = browserServer;

	await fs.mkdirp(DIR);
	await fs.writeFile(path.join(DIR, "wsEndpoint"), browserServer.wsEndpoint());

	const tempDir = path.resolve(__dirname, "../temp");
	if (await fs.exists(tempDir)) {
		await fs.remove(tempDir);
	}
	await fs.copy(path.resolve(__dirname, "../fixtures"), tempDir);
}
