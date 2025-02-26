const path = require("path");
const { createWatchCase } = require("../../dist");
const tempDir = path.resolve(__dirname, `../js/temp`);

/* start each case */
createWatchCase($name$, $src$, $dist$, path.join(tempDir, $name$));
/* end each case */
