import { createRequire } from "node:module";

const exportedRequire = createRequire(import.meta.url);

export { exportedRequire };
