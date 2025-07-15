import update from "../../update";
import { load } from "./parent-file";

it("should bubble update from a nested dependency", () => {
	return load().then(value => {
		expect(value).toBe(1);
		return new Promise((resolve, reject) => {
			module.hot.accept("./parent-file", () => {
				resolve(load().then(value => {
					expect(value).toBe(2);
				}));
			});
			NEXT(update(reject));
		});
	})
});
