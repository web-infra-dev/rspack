const emptyModule = "module.js";
import("./empty/" + emptyModule);

import(`./sub/${"a"}`);
import(`./sub/${"a"}${1}`);
import("./sub/" + "a");
import("./sub/" + "a" + 1);
import("./sub/".concat("a"));
import("./sub/".concat("a", 1));

const evaluateModule = "a";
import(`./sub/${evaluateModule}`);
import(`./sub/${evaluateModule}bc`);
import("./sub/" + evaluateModule);
import("./sub/" + evaluateModule + "");
import("./sub/" + evaluateModule + "bc");
import("./sub/".concat(evaluateModule));
import("./sub/".concat(evaluateModule).concat(""));
import("./sub/".concat(evaluateModule).concat("bc"));
// import("./sub/".concat(testFileName).concat("?queryString"))

const fakeMapModule = "module";
import(`./fake-map/${fakeMapModule}`);
import(`./fake-map/${fakeMapModule}2`);
