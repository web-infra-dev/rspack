const emptyModule = "module.js";
require("./empty/" + emptyModule);

require(`./sub/${"a"}`);
require(`./sub/${"a"}${1}`);
require("./sub/" + "a");
require("./sub/" + "a" + 1);
require("./sub/".concat("a"));
require("./sub/".concat("a", 1));

const evaluateModule = "a";
require(`./sub/${evaluateModule}`);
require(`./sub/${evaluateModule}bc`);
require("./sub/" + evaluateModule);
require("./sub/" + evaluateModule + "");
require("./sub/" + evaluateModule + "bc");
require("./sub/".concat(evaluateModule));
require("./sub/".concat(evaluateModule).concat(""));
require("./sub/".concat(evaluateModule).concat("bc"));
// require("./sub/".concat(testFileName).concat("?queryString"))

const fakeMapModule = "module";
require(`./fake-map/${fakeMapModule}`);
require(`./fake-map/${fakeMapModule}2`);
