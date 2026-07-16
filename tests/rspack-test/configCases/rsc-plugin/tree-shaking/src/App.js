"use server-entry";

import { TodoItem, Todos } from './Todos';

export const App = async () => {
    const { DynamicTodo } = await import('./DynamicTodo');

    return (
        <>
            <Todos />
            <TodoItem />
            <DynamicTodo />
        </>
    );
};
