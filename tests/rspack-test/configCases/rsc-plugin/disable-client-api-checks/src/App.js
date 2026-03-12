// This is a server component (on the RSC layer, without "use client").
// It imports `useState` from React, which is a client-only API.
// Normally this would cause a compile-time error, but with
// `disableClientApiChecks: true` the build should succeed.

import { useState } from 'react';

export const App = () => {
    // In practice this would not work at runtime in a server component,
    // but the compile-time check is intentionally disabled.
    const [count] = useState(0);
    return <div>{count}</div>;
};
