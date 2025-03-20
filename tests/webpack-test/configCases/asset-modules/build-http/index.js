const urlSvg = new URL(
	"https://raw.githubusercontent.com/webpack/webpack/refs/heads/main/test/configCases/asset-modules/_images/file.svg",
	import.meta.url
);
import urlSvg from "https://raw.githubusercontent.com/webpack/webpack/refs/heads/main/test/configCases/asset-modules/_images/file.svg";

it("should work", () => {
	expect(/[\da-f]{20}\.svg$/.test(urlSvg)).toBe(true);
});

it("should bundle the content", () => {
	expect(urlSvg).toContain("<svg");
});
