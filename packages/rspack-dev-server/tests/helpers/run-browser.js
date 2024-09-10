"use strict";

const puppeteer = require("puppeteer");
const { puppeteerArgs } = require("./puppeteer-constants");

/**
 * @typedef {Object} RunBrowserResult
 * @property {import('puppeteer').Page} page
 * @property {import('puppeteer').Browser} browser
 */

/**
 * @param {Parameters<import('puppeteer').Page['emulate']>[0]} config
 * @returns {Promise<RunBrowserResult>}
 */
function runBrowser(config) {
	return new Promise((resolve, reject) => {
		/**
		 * @type {import('puppeteer').Page}
		 */
		let page;
		/**
		 * @type {import('puppeteer').Browser}
		 */
		let browser;

		puppeteer
			.launch({
				headless: "new",
				// because of invalid localhost certificate
				acceptInsecureCerts: true,
				// args come from: https://github.com/alixaxel/chrome-aws-lambda/blob/master/source/index.js
				args: puppeteerArgs
			})
			.then(launchedBrowser => {
				browser = launchedBrowser;

				return runPage(launchedBrowser, config);
			})
			.then(newPage => {
				page = newPage;

				resolve({ page, browser });
			})
			.catch(reject);
	});
}

function runPage(browser, config) {
	/**
	 * @type {import('puppeteer').Page}
	 */
	let page;

	const options = {
		viewport: {
			width: 500,
			height: 500
		},
		userAgent: "",
		...config
	};

	return Promise.resolve()
		.then(() => browser.newPage())
		.then(newPage => {
			page = newPage;
			page.emulate(options);

			return page.setRequestInterception(true);
		})
		.then(() => {
			page.on("request", interceptedRequest => {
				if (interceptedRequest.isInterceptResolutionHandled()) return;
				if (interceptedRequest.url().includes("favicon.ico")) {
					interceptedRequest.respond({
						status: 200,
						contentType: "image/png",
						body: "Empty"
					});
				} else {
					interceptedRequest.continue(
						interceptedRequest.continueRequestOverrides(),
						10
					);
				}
			});

			return page;
		});
}

module.exports = runBrowser;
module.exports.runPage = runPage;
