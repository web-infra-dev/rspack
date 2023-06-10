import { ChangeDetectionStrategy, Component } from '@angular/core';

import { demoComponentContent } from './tooltip-section.list';
import { ContentSection } from '../../common-docs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'tooltip-section',
  templateUrl: './tooltip-section.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class TooltipSectionComponent {
  name = 'Tooltip';
  src = 'https://github.com/valor-software/ngx-bootstrap/tree/development/src/tooltip';
  componentContent: ContentSection[] = demoComponentContent;
}
