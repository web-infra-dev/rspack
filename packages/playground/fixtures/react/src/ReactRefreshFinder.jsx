import React from 'react';

export async function ReactRefreshFinder() {
    async function b() {
        const a = await 1;
        return function RootApp() {
            const [data, setData] = useState(a);
            return <div />
        };
    };
    await b();
}

ReactRefreshFinder()