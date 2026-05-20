import fooStyle from "./foo.css";

it("should handle HMR for exportType", async () => {
	expect(fooStyle).toContain("foo");
	await NEXT_HMR();
	expect(fooStyle).toContain("foo");
});

module.hot.accept(["./foo.css"]);
