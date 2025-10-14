import './change';
import './no-change';

it("css recovery cacheable", done => {
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(
			err => {
				expect(String(err)).toContain("Module build failed");
				NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => done()));
			},
		)
	);
});
