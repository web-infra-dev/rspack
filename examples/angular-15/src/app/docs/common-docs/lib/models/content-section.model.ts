import { Injectable } from '@angular/core';
import { ComponentExample } from './components-examples.model';
import { ComponentApi } from './components-api.model';
import { SourceCodeModel } from "./source-code.model";

@Injectable({providedIn: 'platform'})
export class ContentSection {
  name?: string;
  anchor?: string;
  outlet: any;
  description?: string;
  content?: ComponentExample[] | ComponentApi[];
  importInfo?: string;
  tabName?: 'overview' | 'api' | 'examples';
  usage?: SourceCodeModel;
}
