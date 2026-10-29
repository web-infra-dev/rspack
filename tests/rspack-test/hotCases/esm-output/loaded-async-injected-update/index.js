import store from "./store";
import { load } from "./importer";

export function loadOther() {
	return import("./other");
}

it("should keep updating a module an update injected from an unloaded chunk", (done) => {
	load()
		.then(() => {
			expect(store.value).toBe(1);
			NEXT_HMR().then(() => {
					expect(store.value).toBe("shared");
					NEXT_HMR().then(() => {
							expect(store.value).toBe("shared 2");
							done();
						}).catch(done);
				}).catch(done);
		})
		.catch(done);
});
