import * as styles from './index.css';

it("css hmr", (done) => {
	styles;
	NEXT(require("../../update")(done, true, () => {
		done();
	}));
});
