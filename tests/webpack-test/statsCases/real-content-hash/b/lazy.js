import url from "./file.png";
import test from "./module";

console.log(test, url, new URL("file.jpg?query", import.meta.url));
