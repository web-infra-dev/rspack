const urlSvg = new URL(
	"https://raw.githubusercontent.com/web-infra-dev/rspack/55d5d81/packages/rspack-test-tools/tests/configCases/asset/_images/file.jpg",
	import.meta.url
);

it("should work", () => {
	expect(/[\da-f]{16}\.jpg$/.test(urlSvg)).toBe(true);
});
