document.body.id = 'main';
// `shared.js` is also a dynamic entry, but we hit it first via import() —
// so the proxy is factorized with `is_entry = false` on the import path.
import('./shared.js');
