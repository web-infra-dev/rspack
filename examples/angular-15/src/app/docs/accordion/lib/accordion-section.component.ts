// todo: add more samples https://jqueryui.com/accordion/#default
// todo: add more samples https://getbootstrap.com/docs/3.3/components/#panels-alternatives

import { ChangeDetectionStrategy, Component } from '@angular/core';
import { demoComponentContent } from './accordion-section.list';
import { ContentSection } from '../../common-docs';

// webpack html imports
@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'accordion-section',
  templateUrl: './accordion-section.components.html',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class AccordionSectionComponent {
  name = 'Accordion';
  src = 'https://github.com/valor-software/ngx-bootstrap/tree/development/src/accordion';
  componentContent: ContentSection[] = demoComponentContent;
}
