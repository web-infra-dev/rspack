const fullSync = await import.defer("./full-sync.js");
const asyncMod = await import.defer("./async-mod.js");
const deepAsync = await import.defer("./deep-async.js");
const reexportAsync = await import.defer("./reexport-async.js");

__configCases__defer_import_async_in_graph_dynamic_import.push("START entry.js");

export default { fullSync, asyncMod, deepAsync, reexportAsync };

__configCases__defer_import_async_in_graph_dynamic_import.push("END entry.js");
