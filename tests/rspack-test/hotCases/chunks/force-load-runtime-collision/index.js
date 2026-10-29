import * as shared from "./shared";

if (module.hot) module.hot.accept("./shared");

it("should merge force-load chunks across colliding runtime updates", (done) => {
	expect(shared.value).toBe(1);
	NEXT_HMR().then(() => {
			expect(shared.value).toBe(2);
			done();
		}).catch(done);
});
