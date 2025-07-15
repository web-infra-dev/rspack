// harmony module

import * as fs2 from "./fs";
// import from CommonJS module
import fs, { readFile } from "./fs";

fs.readFile("file");
readFile("file");
fs2.readFile("file");

// import from harmony module
import { readFile as readFile2 } from "./reexport-commonjs";

readFile2("file");

// import a CommonJs module for side effects
import "./example2";
