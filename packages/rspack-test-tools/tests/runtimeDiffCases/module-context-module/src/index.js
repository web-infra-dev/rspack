const m = "module.js";
import("./" + m);
import("./empty/" + m);
import(`./${m}`);

import(`./sub/${"a"}`);
import(`./sub/${"a"}${1}`);
import("./sub/" + "a");
import("./sub/" + "a" + 1);
import("./sub/".concat("a"));
import("./sub/".concat("a", 1));

const testFileName = "a";
import(`./sub/${testFileName}`);
import(`./sub/${testFileName}bc`);
import("./sub/" + testFileName);
import("./sub/" + testFileName + "");
import("./sub/" + testFileName + "bc");
import("./sub/".concat(testFileName));
import("./sub/".concat(testFileName).concat(""));
import("./sub/".concat(testFileName).concat("bc"));
// import("./sub/".concat(testFileName).concat("?queryString"))
