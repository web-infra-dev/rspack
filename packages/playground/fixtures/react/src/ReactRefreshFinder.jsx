import React from 'react';

export function ReactRefreshFinder() {
    return function Component() {
        inner();
        return <div id="nest-function">nest-function</div>;
    };
}

function inner() {}