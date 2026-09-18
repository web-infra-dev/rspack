import {
  createCompiler,
  buildAndCheckLoaders,
} from "../libuv-handles/compiler.mjs";

export default async function run() {
  const { compiler, loaderState } = createCompiler(4);
  // Retain an unclosed compiler: a pending channel receive must not keep Node alive.
  globalThis.idleLoaderCompiler = compiler;
  await buildAndCheckLoaders(compiler, loaderState, 4, false);
}
