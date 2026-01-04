import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  type Argv,
  checkCancel,
  create,
  type ESLintTemplateName,
  select,
} from 'create-rstack';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function getTemplateName({ template }: Argv) {
  if (typeof template === 'string') {
    const pair = template.split('-');
    const language = pair[1] ?? 'js';
    const framework = pair[0];
    return `${framework}-${language}`;
  }

  const framework = checkCancel<string>(
    await select({
      message: 'Select framework',
      options: [
        { value: 'vanilla', label: 'Vanilla' },
        { value: 'react', label: 'React' },
        { value: 'vue', label: 'Vue' },
      ],
    }),
  );

  const language = checkCancel<string>(
    await select({
      message: 'Select language',
      options: [
        { value: 'ts', label: 'TypeScript' },
        { value: 'js', label: 'JavaScript' },
      ],
    }),
  );

  return `${framework}-${language}`;
}

function mapESLintTemplate(templateName: string): ESLintTemplateName {
  switch (templateName) {
    case 'react-js':
    case 'react-ts':
    case 'vue-js':
    case 'vue-ts':
      return templateName;
  }
  const language = templateName.split('-')[1];
  return `vanilla-${language}` as ESLintTemplateName;
}

create({
  root: path.resolve(__dirname, '..'),
  name: 'rspack',
  templates: [
    'vanilla-js',
    'vanilla-ts',
    'react-js',
    'react-ts',
    'vue-js',
    'vue-ts',
  ],
  skipFiles: ['.npmignore'],
  getTemplateName,
  mapESLintTemplate,
});
