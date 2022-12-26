import puppeteer from "puppeteer";
import type { Browser, Page } from "puppeteer";

const puppeteerArgs = [
	"--disable-background-timer-throttling",
	"--disable-breakpad",
	"--disable-client-side-phishing-detection",
	"--disable-cloud-import",
	"--disable-default-apps",
	"--disable-dev-shm-usage",
	"--disable-extensions",
	"--disable-gesture-typing",
	"--disable-hang-monitor",
	"--disable-infobars",
	"--disable-notifications",
	"--disable-offer-store-unmasked-wallet-cards",
	"--disable-offer-upload-credit-cards",
	"--disable-popup-blocking",
	"--disable-print-preview",
	"--disable-prompt-on-repost",
	"--disable-setuid-sandbox",
	"--disable-speech-api",
	"--disable-sync",
	"--disable-tab-for-desktop-share",
	"--disable-translate",
	"--disable-voice-input",
	"--disable-wake-on-wifi",
	"--enable-async-dns",
	"--enable-simple-cache-backend",
	"--enable-tcp-fast-open",
	"--enable-webgl",
	"--hide-scrollbars",
	"--ignore-gpu-blacklist",
	"--media-cache-size=33554432",
	"--metrics-recording-only",
	"--mute-audio",
	"--no-default-browser-check",
	"--no-first-run",
	"--no-pings",
	"--no-sandbox",
	"--no-zygote",
	"--password-store=basic",
	"--prerender-from-omnibox=disabled",
	"--use-gl=swiftshader",
	"--use-mock-keychain"
];

async function runBrowser(
	config?: any
): Promise<{ page: Page; browser: Browser }> {
	const options = {
		viewport: {
			width: 500,
			height: 500
		},
		userAgent: "",
		...config
	};

	return new Promise((resolve, reject) => {
		let page: Page;
		let browser: Browser;

		puppeteer
			.launch({
				headless: true,
				ignoreHTTPSErrors: true,
				args: puppeteerArgs
			})
			.then(launchedBrowser => {
				browser = launchedBrowser;

				return browser.newPage();
			})
			.then(newPage => {
				page = newPage;
				page.emulate(options);

				resolve({ page, browser });
			})
			.catch(reject);
	});
}

export default runBrowser;
