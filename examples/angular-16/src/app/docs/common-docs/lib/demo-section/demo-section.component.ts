import { Component, Input } from '@angular/core';
import { ComponentExample } from '../models/components-examples.model';
import { ComponentApi } from '../models/components-api.model';

interface IContentSection {
  name?: string;
  anchor?: string;
  outlet: any;
  description?: string;
  content?: ComponentExample[] | ComponentApi[];
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-section',
  templateUrl: './demo-section.component.html'
})
export class DemoSectionComponent {
  @Input() name?: string;
  @Input() src?: string;
  @Input() html?: string;
  @Input() ts?: string;

  @Input() componentContent?: IContentSection[];
}
