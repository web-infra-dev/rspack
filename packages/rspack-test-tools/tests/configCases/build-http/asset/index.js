const urlSvg = new URL(
	"https://raw.githubusercontent.com/web-infra-dev/rspack/55d5d812172f6c27d5c454e31360d8719d85d6a7/packages/rspack-test-tools/tests/configCases/asset/_images/file.jpg",
	import.meta.url
);

it("should work", () => {
	expect(/[\da-f]{16}\.jpg$/.test(urlSvg)).toBe(true);
});
