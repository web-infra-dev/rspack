import './change';
import './no-change';

it("css recovery cacheable", async () => {
	try {
		await NEXT_HMR();
	} catch (err) {
		expect(String(err)).toContain("Module build failed");
		await NEXT_HMR();
	}
});

module.hot.accept("./change");
