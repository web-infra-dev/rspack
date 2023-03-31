import { BrowserServer, Page } from "playwright-chromium";

declare global {
	var browserServer: BrowserServer;
	var page: Page;
}
