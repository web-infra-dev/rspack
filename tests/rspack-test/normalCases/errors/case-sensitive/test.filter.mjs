import { platform } from "node:os";
export default () => platform() !== "linux";
