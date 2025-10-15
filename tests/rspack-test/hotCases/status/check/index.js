import { value } from "./file";

it("call module.check api with false should return updatedModules correctly", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(1);
	NEXT(require("./update")(done));
}));
