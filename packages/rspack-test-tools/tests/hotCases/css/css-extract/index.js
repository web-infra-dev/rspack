import * as styles from './index.css';

it("css hmr", (done) => {
	console.log(styles);
	NEXT(require("../../update")(done, true, () => {
		console.log(styles);
		done();
	}));
});
