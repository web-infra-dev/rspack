import defer * as fullSync from "./full-sync.js";
import defer * as asyncMod from "./async-mod.js";
import defer * as deepAsync from "./deep-async.js";
import defer * as reexportAsync from "./reexport-async.js";

__configCases__deferImport__proposal.push("START entry.js");

export default { fullSync, asyncMod, deepAsync, reexportAsync };

__configCases__deferImport__proposal.push("END entry.js");
