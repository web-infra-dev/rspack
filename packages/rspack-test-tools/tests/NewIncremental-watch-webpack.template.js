// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { createWatchNewIncrementalCase } = require("../../dist");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`);
}

const tempDir = path.resolve(
	__dirname,
	`../js/new-incremental/webpack-test/temp`
);

/* start each case */
createWatchNewIncrementalCase($name$, $src$, $dist$, path.join(tempDir, $name$));
/* end each case */
