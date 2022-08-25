import path from 'path';
import { RspackOptions } from "./config";

export function loadConfigFile(root = process.cwd(), configFileName = "rspack.config.js"): RspackOptions {
  // TODO: resolve .ts, .mjs...
  return require(path.resolve(root, configFileName));
}