import "./lib";
import React from "https://esm.sh/react";

console.log('React object imported from HTTP:', React);
console.log('React version:', React.version);
console.log('React createElement is a function:', typeof React.createElement === 'function');

// Try using React
const element = React.createElement('div', { className: 'test' }, 'Hello, world!');
console.log('Created element:', element);
