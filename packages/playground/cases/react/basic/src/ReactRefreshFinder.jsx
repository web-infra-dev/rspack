import React, { useState } from 'react';

function CreateReactRefreshFinder() {
    return function Component() {
        useState(1);
        return <div id="nest-function">nest-function</div>;
    };
}

export const ReactRefreshFinder = CreateReactRefreshFinder()