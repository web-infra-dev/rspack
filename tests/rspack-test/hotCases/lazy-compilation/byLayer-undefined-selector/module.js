import { join } from "bridge";

export const value = join("a", "b").replace(/\\/g, "/");
