const path = require("path");

module.exports = async function loader() {
  const result = await this.importModule(
    path.resolve(__dirname, "./execute-module.js"),
  );
  return `export default ${JSON.stringify(result)}`;
};
