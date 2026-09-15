// Presence of this file makes the dynamic entry function emit `shared`
// as an entry. Removing it triggers BuildEntryAndClean to revoke that
// entry dep, but the import() in main.js still references shared.js.
