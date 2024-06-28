"use strict";

const [, , dir, bin] = process.argv;
process.argv.splice(1, 2);
process.chdir(dir);

require(bin);
