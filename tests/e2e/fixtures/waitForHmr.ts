import { Page } from "@playwright/test";

/**
 * we enable lazy compilation by default, so we need to wait for the page to load
 * 
 * This function waits for the HMR (Hot Module Replacement) response to ensure that the page is updated correctly.
 * It waits for a response that includes "hot-update" in the URL, with a timeout of 5000 milliseconds,
 * and then waits for an additional 500 milliseconds to ensure the page has time to reflect the changes.
 * @param page 
 */
export async function waitForHmr(page: Page) {
  await page.waitForResponse(response => response.url().includes("hot-update"), { timeout: 5000 });
  await page.waitForTimeout(500);
}