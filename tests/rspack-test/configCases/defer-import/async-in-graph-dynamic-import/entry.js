const fullSync = await import.defer("../async-in-graph/full-sync.js");
const asyncMod = await import.defer("../async-in-graph/async-mod.js");
const deepAsync = await import.defer("../async-in-graph/deep-async.js");
const reexportAsync = await import.defer("../async-in-graph/reexport-async.js");

__configCases__deferImport__proposal.push("START entry.js");

export default { fullSync, asyncMod, deepAsync, reexportAsync };

__configCases__deferImport__proposal.push("END entry.js");
