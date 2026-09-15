import { value as shared } from "./shared";
import { trace } from "./trace";

trace.push("first");
export const value = `${shared}:first`;
