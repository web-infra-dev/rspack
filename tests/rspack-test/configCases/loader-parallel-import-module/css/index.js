import stylesheet from "./stylesheet";
import otherStylesheet from "./other-stylesheet";

it("should be able to use build-time code", () => {
	expect(stylesheet).toBe(
		'body { background: url("my-schema://base/public/assets/file.png"); color: #f00; }'
	);
	expect(otherStylesheet).toBe(
		'body { background: url("/other/assets/file.jpg"); color: #0f0; }'
	);
});
