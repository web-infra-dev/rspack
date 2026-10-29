import store from "./store";
import "./mod";

it("should load a new initial chunk before re-executing the updated module", (done) => {
	expect(store.value).toBe(1);
	NEXT_HMR().then(() => {
			expect(store.value).toBe("dep");
			done();
		}).catch(done);
});
