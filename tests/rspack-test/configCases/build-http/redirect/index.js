import url from 'https://github.com/web-infra-dev/rspack/raw/main/packages/rspack-test-tools/tests/configCases/asset/_images/file.png';

it("should work", () => {
    expect(/[\da-f]{16}\.png$/.test(url)).toBe(true);
});
