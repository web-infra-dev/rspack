const fs = require("fs");

module.exports = function (source) {
  const depPath = this.resource.replace("entry.txt", "dep.txt");
  this.addDependency(depPath);
  const depContent = fs.readFileSync(depPath, 'utf-8');
  if (depContent === "fail") {
    throw new Error("Failed");
  }
  return `.my-class {
    ${source.replace("$dep$", depContent)}
  }`;
};