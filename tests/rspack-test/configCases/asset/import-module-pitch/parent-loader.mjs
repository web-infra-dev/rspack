import { fileURLToPath } from "node:url";

export default async function () {
  const { default: url } = await this.importModule(
    `!!${fileURLToPath(import.meta.resolve('./pitch-loader.mjs'))}!${fileURLToPath(import.meta.resolve('./target.txt'))}`,
    { publicPath: '' },
  );
  return `module.exports = ${JSON.stringify(url)};`;
};
