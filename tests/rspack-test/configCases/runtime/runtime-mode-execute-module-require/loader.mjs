import path from "node:path";

export default async function loader() {
  const result = await this.importModule(
    path.resolve(import.meta.dirname, "./execute-module.js"),
  );
  return `export default ${JSON.stringify(result)}`;
};
