'use server-entry';

import './Todos.css';
import { Dialog } from './Dialog';
import { TodoCreate } from './TodoCreate';
import { TodoDetail } from './TodoDetail';
import { TodoList } from './TodoList';

export async function Todos({ id }: { id?: number }) {
  return (
    <html lang="en" style={{ colorScheme: 'dark light' }}>
      <head>
        <title>Todos</title>
      </head>
      <body>
        <p data-testid="remote-app-root">remote-app-root</p>
        <header>
          <h1>Todos</h1>
          <Dialog trigger="+" buttonTestId="remote-app-add-dialog-button">
            <h2>Add todo</h2>
            <TodoCreate />
          </Dialog>
        </header>
        <main>
          <div className="todo-column">
            <TodoList id={id} />
          </div>
          {id != null ? <TodoDetail key={id} id={id} /> : <p>Select a todo</p>}
        </main>
      </body>
    </html>
  );
}
