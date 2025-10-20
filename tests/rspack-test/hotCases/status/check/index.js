import { value } from "./file";

it("call module.check api with false should return updatedModules correctly", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
});

module.hot.accept("./file");
