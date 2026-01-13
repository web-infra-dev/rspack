# Cross-Origin Lazy Compilation Test

This test verifies that lazy compilation works correctly
when the lazy compilation server runs on a different origin (port) than the frontend dev server.

## Architecture

```
+----------------------+    +----------------------+    +-----------------------------+
|                      |    |                      |    |                             |
|      Browser         |    |      Dev Server      |    |   Lazy Compilation Server   |
|      (Frontend)      |    |      (Port: 8500)    |    |        (Port: 8600)          |
|                      |    |                      |    |                             |
+----------+-----------+    +----------+-----------+    +--------------+--------------+
           |                         |                                 |
           |  1. Load page           |                                 |
           |  GET http://localhost:8500                                |
           +------------------------->                                 |
           |                         |                                 |
           |  2. Click button -> dynamic import()                      |
           |                         |                                 |
           |  3. Cross-origin POST request                             |
           |     POST http://127.0.0.1:8600/lazy-...                   |
           |     Content-Type: text/plain                              |
           |     Body: "module-id-1\nmodule-id-2"                      |
           +---------------------------------------------------------->
           |                         |                                 |
           |  4. Response (SSE or empty for POST)                      |
           <----------------------------------------------------------+
           |                         |                                 |
           |  5. Webpack invalidate & rebuild                          |
           |                         |                                 |
           |  6. Load compiled chunk                                   |
           v                         |                                 |
+----------------------+             |                                 |
|      Component       |             |                                 |
|      Rendered        |             |                                 |
+----------------------+             |                                 |
```

## Key Points

1. **Two Separate Servers**: Frontend runs on port 8500, lazy compilation on port 8600
2. **Cross-Origin Request**: Browser sends POST request to a different origin
3. **Simple Request**: Uses `Content-Type: text/plain` to avoid CORS preflight
4. **XMLHttpRequest**: Uses XHR instead of fetch for better browser compatibility

