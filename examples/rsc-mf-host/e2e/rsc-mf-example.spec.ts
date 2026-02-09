import { expect, test } from '@playwright/test';

test('host consumes remote RSC modules and executes remote actions', async ({
  page,
}) => {
  const remoteEntryResponse = await page.request.get(
    'http://localhost:1717/remoteEntry.cjs',
  );
  expect(remoteEntryResponse.ok()).toBeTruthy();

  await page.goto('/');

  await expect(page.getByTestId('mf-source')).toHaveText('remote-http');
  await expect(page.getByTestId('remote-server-only-label')).toHaveText(
    'remote-server-only-ok',
  );
  await expect(page.getByTestId('remote-shell-title')).toHaveText(
    'Remote Todo Shell',
  );
  await expect(page.getByTestId('remote-actions-export-count')).toContainText(
    'remote-actions-exports:',
  );

  const remoteShell = page.getByTestId('remote-shell');
  await remoteShell.getByTestId('remote-add-dialog-button').click();

  const createForm = remoteShell.getByTestId('todo-create-form');
  await createForm.locator('input[name="title"]').fill('mf-rsc-item');
  await createForm
    .locator('textarea[name="description"]')
    .fill('created via remote server action');
  await createForm.locator('input[name="dueDate"]').fill('2030-01-01');
  await createForm.getByTestId('todo-create-submit').click();

  await expect(remoteShell.getByTestId('todo-item-link-0')).toHaveText(
    'mf-rsc-item',
  );

  await remoteShell.getByTestId('todo-item-link-0').click();
  await expect(page).toHaveURL(/\/todos\/0$/);

  const detailForm = remoteShell.getByTestId('todo-detail-form');
  await detailForm.locator('input[name="title"]').fill('mf-rsc-item-updated');
  await detailForm.getByTestId('todo-update-submit').click();

  await expect(remoteShell.getByTestId('todo-item-link-0')).toHaveText(
    'mf-rsc-item-updated',
  );

  await remoteShell.getByTestId('todo-item-checkbox-0').check();
  await remoteShell.getByTestId('todo-item-delete-0').click();
  await expect(remoteShell.getByTestId('todo-item-link-0')).toHaveCount(0);
});
