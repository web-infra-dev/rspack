const fs = require("fs");
const path = require("path");

const outputFile = path.join(__dirname, "runtime-result.json");
fs.writeFileSync(outputFile, JSON.stringify(["second-entry"]));
