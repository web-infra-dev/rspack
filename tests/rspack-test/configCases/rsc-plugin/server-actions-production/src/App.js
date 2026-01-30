import { add, del, get, update } from './actions';

export const App = async () => {
    await add();
    await del();
    await get();
    await update();

    return (
        <h1>RSC App</h1>
    );
};
