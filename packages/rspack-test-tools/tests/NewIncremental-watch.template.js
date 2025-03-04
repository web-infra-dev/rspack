// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { createWatchNewIncrementalCase } = require("../../dist");
const tempDir = path.resolve(__dirname, `../js/new-incremental/temp`);

// Run tests rspack-test-tools/tests/watchCases
/* start each case */
createWatchNewIncrementalCase($name$, $src$, $dist$, path.join(tempDir, $name$));
/* end each case */
