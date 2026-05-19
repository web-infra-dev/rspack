import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  type Argv,
  checkCancel,
  copyFolder,
  create,
  type ESLintTemplateName,
  type RslintTemplateName,
  select,
} from 'create-rstack';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');

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

function mapRstestTemplate(templateName: string): string {
  switch (templateName) {
    case 'react-js':
    case 'react-ts':
    case 'vue-js':
    case 'vue-ts':
      return templateName;
    default:
      return `vanilla-${templateName.split('-')[1]}`;
  }
}

function mapRslintTemplate(templateName: string): RslintTemplateName {
  switch (templateName) {
    case 'react-js':
    case 'react-ts':
      return templateName;
    default:
      return `vanilla-${templateName.split('-')[1]}` as RslintTemplateName;
  }
}

create({
  root,
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
  mapRslintTemplate,
  extraTools: [
    {
      value: 'rstest',
      label: 'Rstest - testing',
      order: 'pre',
      action: ({ templateName, distFolder, addAgentsMdSearchDirs }) => {
        const rstestTemplate = mapRstestTemplate(templateName);
        const toolFolder = path.join(root, 'template-rstest');
        const subFolder = path.join(toolFolder, rstestTemplate);

        copyFolder({
          from: subFolder,
          to: distFolder,
          isMergePackageJson: true,
        });
        addAgentsMdSearchDirs(toolFolder);
      },
    },
  ],
  extraSkills: [
    {
      value: 'rspack-best-practices',
      label: 'Rspack best practices',
      source: 'rstackjs/agent-skills',
    },
    {
      value: 'rstest-best-practices',
      label: 'Rstest best practices',
      source: 'rstackjs/agent-skills',
      when: ({ tools }) => tools.includes('rstest'),
    },
    {
      value: 'vercel-react-best-practices',
      label: 'React best practices',
      source: 'vercel-labs/agent-skills',
      when: ({ templateName }) => templateName.startsWith('react-'),
    },
  ],
});
