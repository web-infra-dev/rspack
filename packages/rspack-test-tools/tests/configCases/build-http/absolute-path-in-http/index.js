import React from 'react';
import { createRoot } from 'react-dom/client';

// Import content that contains absolute paths
import { message, getMessage } from "https://example.com/absolute-path-test.js";
import App from './App';

// Create a container for React rendering if we're in a browser environment
let root;

const renderApp = () => {
  if (typeof document !== 'undefined') {
    const container = document.getElementById('root') || document.createElement('div');
    container.id = 'root';
    if (!document.getElementById('root')) {
      document.body.appendChild(container);
    }

    // Initialize root only once to preserve state during HMR
    if (!root) {
      root = createRoot(container);
    }

    root.render(<App message={message} getMessage={getMessage} />);
  }
};

// Initial render
renderApp();

// Export for testing
export { message, getMessage };

// Tests
it("should correctly import real module using absolute path from HTTP-imported content", () => {
  // Verify that we can access the real module's exports
  expect(message).toBe("Hello from real module!");
  expect(getMessage()).toBe("Hello from real module!");
});

it("should treat absolute paths as local filesystem paths", () => {
  // The imported content should contain absolute paths that should be treated as local paths
  // and not as HTTP URLs
  expect(typeof message).toBe("string");
  expect(typeof getMessage).toBe("function");
});

// Set up hot module replacement
if (module.hot) {
  // Accept updates for the current module and dependencies
  module.hot.accept('./App', () => {
    // When App.jsx changes, re-render the app
    console.log('🔄 Hot-updating App component');
    renderApp();
  });

  // Accept updates for the HTTP module
  module.hot.accept("https://example.com/absolute-path-test.js", () => {
    console.log('🔄 Hot-updating remote module');
    renderApp();
  });
}
