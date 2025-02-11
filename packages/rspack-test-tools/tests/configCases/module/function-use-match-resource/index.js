import { A, B, __info__, __context__ } from "__label__?A=A#B=B!=!./lib";

it("use entry arguments should be correct", () => {
	expect(A).toBe('A');
	expect(B).toBe('B');
	expect(__info__).toMatchInlineSnapshot(`
{
  "resource": "__label__",
  "realResource": "<TEST_TOOLS_ROOT>/tests/configCases/module/function-use-match-resource/lib.js",
  "resourceQuery": "?A=A",
  "resourceFragment": "#B=B",
  "issuer": "<TEST_TOOLS_ROOT>/tests/configCases/module/function-use-match-resource/index.js",
  "issuerLayer": ""
}
`);
});
