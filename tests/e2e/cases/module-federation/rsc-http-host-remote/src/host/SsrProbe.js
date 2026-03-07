import * as React from 'react';

export function readSsrProbe() {
  return typeof React.use;
}
