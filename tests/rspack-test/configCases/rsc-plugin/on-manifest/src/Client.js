'use client';

import './Client.css';
import { add, del, get, update } from './actions';

export const Client = () => {
    async function onClick() {
        await add();
        await del();
        await get();
        await update();
    }

    return (
        <button className="client-button" type="button" onClick={onClick}>
            Run actions
        </button>
    );
};
