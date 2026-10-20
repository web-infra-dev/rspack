// Adapted from webpack/webpack test/configCases/mangle/mangle-escaping-namespace-external (MIT).
import * as ns from "./provider";

// Namespace escapes via function return
export const getProvider = () => ns;
