'use server-entry';

import './Todos.css';
import { Dialog } from './Dialog';
import { getServerOnlyLabel } from './serverOnly';
import { TodoCreate } from './TodoCreate';
import { TodoDetail } from './TodoDetail';
import { TodoList } from './TodoList';

export async function RemoteShell({ id }: { id?: number }) {
  return (
    <section data-testid="remote-shell">
      <header>
        <h1 data-testid="remote-shell-title">Remote Todo Shell</h1>
        <Dialog trigger="+" buttonTestId="remote-add-dialog-button">
          <h2>Add todo</h2>
          <TodoCreate />
        </Dialog>
      </header>
      <p data-testid="remote-server-only">{getServerOnlyLabel()}</p>
      <main>
        <div className="todo-column">
          <TodoList id={id} />
        </div>
        {id != null ? <TodoDetail key={id} id={id} /> : <p>Select a todo</p>}
      </main>
    </section>
  );
}
