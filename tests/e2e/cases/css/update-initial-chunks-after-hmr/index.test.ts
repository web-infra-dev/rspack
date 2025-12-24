import { test, expect } from "@/fixtures";

const COLOR_BLUE = "rgb(163, 255, 255)";
const COLOR_WHITE = "rgb(255, 255, 255)";

test("should update body css", async ({ page, fileAction }) => {
	await expect(page.locator("body")).toHaveCSS("background-color", COLOR_BLUE);

	// trigger css hmr
	fileAction.updateFile("src/index.js", content =>
		content.replace('import "./blue.css";', '// import "./blue.css";')
	);

	await expect(page.locator("body")).toHaveCSS("background-color", COLOR_WHITE);

	// initial css chunk update, without css hmr
	fileAction.updateFile("src/index.js", content => content.replace("//", ""));

	await expect(page.locator("body")).toHaveCSS("background-color", COLOR_BLUE);
});
