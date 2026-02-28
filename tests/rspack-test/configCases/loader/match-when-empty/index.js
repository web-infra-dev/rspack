import a1 from "./a.js" with { type: "raw" };
import a2 from "./a.js"; 

it("should hit loader", () => {
	expect(a1).toEqual("export default \"a.js\"");
	expect(a2).toEqual("loader.js");
});