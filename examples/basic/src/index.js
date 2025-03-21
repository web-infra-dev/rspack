// Import React from HTTP URL
import React from "https://esm.sh/react@18.2.0";

console.log("React version:", React.version);

// Get main app element
const app = document.getElementById("app");

// Setup the app UI
if (app) {
	app.innerHTML = `
    <div>
      <h1>HTTP URI Plugin Test</h1>
      <p>React version: ${React.version}</p>

      <h2>Dynamic Import Test</h2>
      <div id="dynamic-import-result">Loading...</div>

      <h2>Custom HTTP Client Test</h2>
      <div id="custom-http-client-result">
        <p>Number of requests: <span id="request-count">0</span></p>
      </div>
    </div>
  `;
}

// Import our custom HTTP client (before dynamic import so it can be used)
import "./custom-http-client.js";

// Import our dynamic import test
import "./dynamic-import.js";

// Check if custom HTTP client was used
setTimeout(() => {
	const requestCountElement = document.getElementById("request-count");
	if (requestCountElement && window.getRequestCount) {
		requestCountElement.textContent = window.getRequestCount();
	}
}, 2000);
