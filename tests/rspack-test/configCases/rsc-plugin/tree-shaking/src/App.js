"use server-entry";

import { TodoItem, Todos } from './Todos';

export const App = async () => {
    return (
        <>
            <Todos />
            <TodoItem />
        </>
    );
};
