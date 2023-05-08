import { SourceCodeModel } from './source-code.model';

export interface ComponentExample {
  anchor: string;
  title: string;
  description?: string;
  component?: SourceCodeModel;
  html?: SourceCodeModel;
  style?: string;
  css?: string;
  outlet?: any; // ToDo: Component<T>
}
