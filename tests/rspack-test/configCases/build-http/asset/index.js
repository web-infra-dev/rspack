import url1 from "https://raw.githubusercontent.com/web-infra-dev/rspack/55d5d81/packages/rspack-test-tools/tests/configCases/asset/_images/file.png";

const url2 = new URL(
	"https://raw.githubusercontent.com/web-infra-dev/rspack/55d5d81/packages/rspack-test-tools/tests/configCases/asset/_images/file.jpg",
	import.meta.url
);

it("should work", () => {
	expect(/[\da-f]{16}\.png$/.test(url1)).toBe(true);
	expect(/[\da-f]{16}\.jpg$/.test(url2)).toBe(true);
});
