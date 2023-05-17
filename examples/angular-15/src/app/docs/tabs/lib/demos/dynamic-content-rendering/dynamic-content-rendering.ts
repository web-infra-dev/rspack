import { ChangeDetectionStrategy, Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'dynamic-content-rendering',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './dynamic-content-rendering.html',
  styleUrls: ['./dynamic-content-rendering.scss']
})
export class DynamicContentRenderingComponent {

  messages: string[] = [];

  message(s: string) {
    this.messages.push(s);
  }

}
