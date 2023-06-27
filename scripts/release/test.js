import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __filename = resolve(fileURLToPath(import.meta.url));
const __dirname = dirname(__filename)
console.log(__dirname)

