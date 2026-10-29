import store from "./store";
import "./holder";

module.hot.accept("./holder");

it("should load a new dependency of a cached module that became async-only", (done) => {
	expect(store.value).toBe(1);
	NEXT_HMR().then(() => {
			expect(store.value).toBe(1);
			NEXT_HMR().then(() => {
					expect(store.value).toBe(3);
					done();
				}).catch(done);
		}).catch(done);
});
