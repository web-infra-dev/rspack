import './change';
import './no-change';

it("css recovery cacheable", done => {
	NEXT(
		require("../../update")(
			err => {
				expect(String(err)).toContain("Module build failed");
				NEXT(require("../../update")(done, true, () => done()));
			},
		)
	);
});
