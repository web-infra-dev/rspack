'use client';

import { add, del, get, update } from './actions';
import './Client.css';

export const Client = () => {
    async function onClick() {
        await add();
        await del();
        await get();
        await update();
    }

    return (
        <button className="client-action" type="button" onClick={onClick}>
            Run actions
        </button>
    );
};
