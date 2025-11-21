import { expect, test } from "@/fixtures";

function has_dyn_module(modules: string[]) {
	for (const m of modules) {
		if (m.endsWith("/src/dyn.js") && !m.startsWith("lazy-compilation-proxy|")) {
			return true;
		}
	}
	return false;
}

test("should load success", async ({ page, rspack }) => {
	// rspack.compiler.__modules is injected by plugin in rspack.config.js
	await expect(page.locator("#Component")).toHaveCount(1);
	await expect(page.locator("#dyn")).toHaveCount(0);
	expect(has_dyn_module(rspack.compiler.__modules)).toBe(false);
	await page.locator("#click_button").click();
	await expect(page.locator("#dyn")).toHaveCount(1);
	expect(has_dyn_module(rspack.compiler.__modules)).toBe(true);

	// trigger other import compile
	await rspack.reboot();

	await expect(page.locator("#Component")).toHaveCount(1);
	await expect(page.locator("#dyn")).toHaveCount(0);
	expect(has_dyn_module(rspack.compiler.__modules)).toBe(true);
	await page.locator("#click_button").click();
	await expect(page.locator("#dyn")).toHaveCount(1);
	expect(has_dyn_module(rspack.compiler.__modules)).toBe(true);
});
