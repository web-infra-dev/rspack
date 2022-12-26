import React from 'react';
import {createRoot } from 'react-dom/client';
import { App } from './app';
const container = createRoot(document.getElementById('root'));
container.render(React.createElement(App));
