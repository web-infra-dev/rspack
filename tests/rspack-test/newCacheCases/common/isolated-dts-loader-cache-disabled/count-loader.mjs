import fs from "node:fs";

export default function (source, sourceMap, additionalData) {
  fs.appendFileSync(this.getOptions().counterFile, "1");
  this.callback(null, source, sourceMap, additionalData);
};
