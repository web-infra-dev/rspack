import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

const base = "http://localhost:3001";

const demoPages = [
	{ name: "Main", hash: "#/" },
	{ name: "UI Library", hash: "#/ui-library" },
	{ name: "Dialog", hash: "#/dialog" },
	{ name: "Svelte Page", hash: "#/svelte" },
	{ name: "Routing", hash: "#/routing/foo" }
];

const appLinks = [
	{ name: "App #1", href: "http://localhost:3001" },
	{ name: "App #2", href: "http://localhost:3002" },
	{ name: "App #3", href: "http://localhost:3003" },
	{ name: "App #4", href: "http://localhost:3004" }
];

const mainPageParagraphs = [
	"Welcome to the Module Federation Demo!",
	"Click any of the items on the left to get started.",
	"Feel free to leave me feedback"
];

const uiLibraryParagraphs = [
	"Simple example showing host app and external component using separate CSS solutions.",
	"This Button component can be found in App #3.",
	"This button is also used in the routing demo."
];

const routingParagraphs = [
	'The following tab components are being imported remotely from "bravo-app".',
	"Notice that your browser's route is /routing/<foo|bar> depending on which tab is active.",
	"If you open http://localhost:3002 you will see the same tab components at the root level",
	'The "Bar" tab also lazily renders the styled-component Button from the UI Library demo only when rendered.'
];

const expectAppBar = async (page: Page, title: string) => {
	const appBar = page.locator("header").first();
	await expect(appBar).toBeVisible();
	await expect(appBar).toHaveCSS("background-color", "rgb(25, 118, 210)");
	await expect(page.getByRole("heading", { name: title })).toBeVisible();
};

test.describe("Comprehensive Demo App1", () => {
	test("main page displays sidebar links and elements", async ({ page }) => {
		await page.goto(base);

		await expect(page.getByRole("heading", { name: "SideNav" })).toBeVisible();
		await expect(page.getByText("Demo Pages")).toBeVisible();
		await expect(page.getByText("Apps")).toBeVisible();

		for (const { name, hash } of demoPages) {
			const link = page.locator("a", { hasText: name }).first();
			await expect(link).toBeVisible();
			await expect(link).toHaveAttribute("href", hash);
		}

		for (const { name, href } of appLinks) {
			const link = page.locator(`a[href="${href}"]`).first();
			await expect(link).toBeVisible();
			await expect(link).toHaveAttribute("href", href);
			await expect(link).toContainText(name);
			await expect(link).toContainText(href);
		}

		await expectAppBar(page, "Module Federation Demo");

		for (const paragraph of mainPageParagraphs) {
			await expect(page.locator("p", { hasText: paragraph })).toBeVisible();
		}

		await expect(
			page.getByRole("link", {
				name: "https://github.com/module-federation/mfe-webpack-demo"
			})
		).toHaveAttribute(
			"href",
			"https://github.com/module-federation/mfe-webpack-demo"
		);
	});

	test("main tab functionality", async ({ page }) => {
		await page.goto(base);

		for (const { name, hash } of demoPages) {
			await page.locator("a", { hasText: name }).first().click();
			await expect(page).toHaveURL(`${base}/${hash}`);
		}

		await page.locator("a", { hasText: "Main" }).first().click();
		await expect(page).toHaveURL(`${base}/#/`);

		for (const { href } of appLinks) {
			const response = await page.request.get(href);
			expect(response.ok()).toBeTruthy();
		}
	});

	test("UI library page renders remote button", async ({ page }) => {
		await page.goto(`${base}/#/ui-library`);

		await expectAppBar(page, "UI Library Demo");

		for (const paragraph of uiLibraryParagraphs) {
			await expect(page.locator("p", { hasText: paragraph })).toBeVisible();
		}

		await expect(
			page.locator('a[href="http://localhost:3003/"]').first()
		).toHaveAttribute("href", "http://localhost:3003/");
		await expect(
			page.locator('a[href="http://localhost:3001/#/routing/foo"]').first()
		).toHaveAttribute("href", "http://localhost:3001/#/routing/foo");

		const styledButton = page.getByRole("button", { name: "💅 Button" });
		await expect(styledButton).toBeVisible();
		await expect(styledButton).toHaveCSS(
			"background-color",
			"rgb(219, 112, 147)"
		);
	});

	test("dialog page loads and dialog opens", async ({ page }) => {
		await page.goto(`${base}/#/dialog`);

		await expectAppBar(page, "Dialog Demo");
		await expect(
			page.locator("p", {
				hasText:
					"Clicking the button below will render a Dialog using React Portal. This dialog component is being lazy loaded from the app #2."
			})
		).toBeVisible();

		await page.getByRole("button", { name: "Open Dialog" }).click();
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
	});

	test("svelte page updates greeting", async ({ page }) => {
		await page.goto(`${base}/#/svelte`);

		await expectAppBar(page, "Svelte Demo");

		const input = page.locator("input");
		await expect(input).toBeVisible();
		await input.fill("May The Force Be With You");
		await expect(page.locator("h1")).toHaveText(
			"Hello From Svelte May The Force Be With You!"
		);
	});

	test("routing page renders tabs", async ({ page }) => {
		await page.goto(`${base}/#/routing/foo`);

		await expectAppBar(page, "Routing Demo");

		for (const paragraph of routingParagraphs) {
			await expect(page.locator("p", { hasText: paragraph })).toBeVisible();
		}

		await expect(page.getByRole("tab", { name: "Foo" })).toBeVisible();
		await expect(page.getByText("Foo Content")).toBeVisible();

		await page.getByRole("tab", { name: "Bar" }).click();
		await expect(page.getByText("Bar Content")).toBeVisible();

		const barButton = page.getByRole("button", { name: "Bar Button" });
		await expect(barButton).toBeVisible();
		await expect(barButton).toHaveCSS("background-color", "rgb(219, 112, 147)");
	});
});
