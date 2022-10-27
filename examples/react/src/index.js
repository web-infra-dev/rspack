const React = require('react');
const {createRoot} = require('react-dom/client')
const { App } = require('./app.jsx')
const container = createRoot(document.getElementById('root'));
container.render(React.createElement(App));
