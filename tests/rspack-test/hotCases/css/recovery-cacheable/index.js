import './change';
import './no-change';

it("css recovery cacheable", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	NEXT(
		require("../../update")(
			err => {
				expect(String(err)).toContain("Module build failed");
				NEXT(require("../../update")(done, true, () => done()));
			},
		)
	);
}));
