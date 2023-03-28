//@ts-nocheck
import os from "os";
import fs from "fs";
import path from "path";
import NodeEnvironment from "jest-environment-node";
import { chromium } from "playwright-chromium";

const DIR = path.join(os.tmpdir(), "jest_playwright_ws_endpoint");

module.exports = class PlaywrightEnvironment extends NodeEnvironment {
	constructor(config, context) {
		super(config);
		this.testPath = context.testPath;
	}

	async setup() {
		await super.setup();
		const wsEndpoint = fs.readFileSync(path.join(DIR, "wsEndpoint"), "utf-8");
		this.browser = await chromium.connect(wsEndpoint);
		this.global.page = await this.browser.newPage();
	}

	async teardown() {
		if (this.browser) {
			await this.browser.close();
		}
		await super.teardown();
	}
};
