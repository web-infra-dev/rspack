import React, { setVersion } from 'react';

// This file will be processed by the layered loader
export default function initializeLayeredReactVersion() {
  // Set the layered React version
  setVersion('1.2.3');
}

// Initialize version immediately
initializeLayeredReactVersion();
