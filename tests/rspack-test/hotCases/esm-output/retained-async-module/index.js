import store from "./store";
import "./holder";

import.meta.webpackHot.accept("./holder");

it("should keep updating a cached module that became async-only", (done) => {
	expect(store.value).toBe(1);
	NEXT_HMR().then(() => {
			expect(store.value).toBe(1);
			NEXT_HMR().then(() => {
					expect(store.value).toBe(3);
					done();
				}).catch(done);
		}).catch(done);
});
