import fooStyle from "./foo.css";

it("should handle HMR for exportType", async () => {
	expect(fooStyle).toContain("bar-v1");
	module.hot.accept(["./foo.css"]);

	await NEXT_HMR();

	expect(fooStyle).toContain("bar-v2");
});
