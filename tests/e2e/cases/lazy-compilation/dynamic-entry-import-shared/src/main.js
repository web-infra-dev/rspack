document.body.dataset.main = '1';
// `shared.js` is also a dynamic entry, but we hit it first via import() —
// so the proxy is factorized with `is_entry = false` on the import path.
import('./shared.js').then(() => {
  document.body.dataset.sharedFromImport = '1';
});
