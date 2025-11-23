import { expect, test } from "@playwright/test";

const base = "http://localhost:3002";

test.describe("Comprehensive Demo App2", () => {
	test("renders blocks, dialog and tabs", async ({ page }) => {
		const events: string[] = [];
		page.on("console", msg =>
			events.push(`console.${msg.type()}: ${msg.text()}`)
		);
		page.on("pageerror", err => events.push(`pageerror: ${err.message}`));
		page.on("requestfailed", req =>
			events.push(
				`requestfailed: ${req.url()} ${req.failure()?.errorText || ""}`.trim()
			)
		);
		page.on("response", res => {
			if (!res.ok()) events.push(`response ${res.status()}: ${res.url()}`);
		});
		try {
			// Clear all browser storage to prevent stale Module Federation metadata
			await page.context().clearCookies();
			await page.goto(base, { waitUntil: "domcontentloaded" });
			// Clear all local storage, caches, and IndexedDB entries for federation data
			await page.evaluate(async () => {
				try {
					localStorage.clear();
					sessionStorage.clear();
				} catch (err) {
					console.warn("[e2e] unable to clear Web Storage", err);
				}
				if ("caches" in window) {
					try {
						const names = await caches.keys();
						await Promise.all(names.map(name => caches.delete(name)));
					} catch (err) {
						console.warn("[e2e] unable to clear Cache API", err);
					}
				}
				if (indexedDB && indexedDB.databases) {
					try {
						const dbs = await indexedDB.databases();
						await Promise.all(
							dbs
								.map(db => db?.name)
								.filter(Boolean)
								.map(name => {
									try {
										const req = indexedDB.deleteDatabase(name);
										return new Promise(resolve => {
											req.onsuccess =
												req.onerror =
												req.onblocked =
													() => resolve(null);
										});
									} catch (err) {
										console.warn("[e2e] deleteDatabase failed", err);
										return Promise.resolve(null);
									}
								})
						);
					} catch (err) {
						console.warn("[e2e] unable to enumerate IndexedDB databases", err);
					}
				}
			});
			// Force hard reload to clear any runtime cache
			await page.reload({ waitUntil: "domcontentloaded" });

			await expect(page.locator("header").first()).toHaveCSS(
				"background-color",
				"rgb(76, 175, 80)" // App 2 theme uses MUI green palette
			);
			await expect(
				page.getByRole("heading", { name: "Material UI App" })
			).toBeVisible();
			await expect(
				page.getByRole("heading", { name: "Dialog Component" })
			).toBeVisible();

			const openDialogButton = page.getByRole("button", {
				name: "Open Dialog"
			});
			await expect(openDialogButton).toBeVisible();
			await openDialogButton.click();

			const dialog = page.locator('[role="dialog"]');
			await expect(
				dialog.getByRole("heading", { name: "Dialog Example" })
			).toBeVisible();
			await expect(
				dialog.getByText(
					"This is a dialog from the Material UI app rendered in a React Portal."
				)
			).toBeVisible();
			await dialog.getByRole("button", { name: "Nice" }).click();
			await expect(dialog).not.toBeVisible();

			await expect(
				page.getByRole("heading", { name: "Tabs Component" })
			).toBeVisible();
			const fooTab = page.getByRole("tab", { name: "Foo" });
			const barTab = page.getByRole("tab", { name: "Bar" });
			await expect(fooTab).toBeVisible();
			await expect(barTab).toBeVisible();
			await expect(page.getByText("Foo Content")).toBeVisible();

			await barTab.click();
			await expect(page.getByText("Bar Content")).toBeVisible();
			await expect(page.getByRole("button", { name: "Bar Button" })).toHaveCSS(
				"background-color",
				"rgb(219, 112, 147)" // Styled-components button from App3
			);
		} catch (e) {
			console.log("[App2 e2e diagnostics]\\n" + events.join("\\n"));
			throw e;
		}
	});
});
