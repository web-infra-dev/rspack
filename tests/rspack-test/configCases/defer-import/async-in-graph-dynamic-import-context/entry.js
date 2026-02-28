let file;
file = "full-sync.js";
const fullSync = await import.defer("./dir/" + file);
file = "async-mod.js";
const asyncMod = await import.defer("./dir/" + file);
file = "deep-async.js";
const deepAsync = await import.defer("./dir/" + file);
file = "reexport-async.js";
const reexportAsync = await import.defer("./dir/" + file);

__configCases__defer_import_async_in_graph_dynamic_import_context.push("START entry.js");

export default { fullSync, asyncMod, deepAsync, reexportAsync };

__configCases__defer_import_async_in_graph_dynamic_import_context.push("END entry.js");
