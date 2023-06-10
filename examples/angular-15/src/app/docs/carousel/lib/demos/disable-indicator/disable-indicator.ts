import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-carousel-disable-indicator',
  templateUrl: './disable-indicator.html'
})
export class DemoCarouselDisableIndicatorComponent {
  slides: {image: string; text?: string}[] =  [
    {image: 'assets/images/nature/5.jpg'},
    {image: 'assets/images/nature/4.jpg'},
    {image: 'assets/images/nature/3.jpg'}
  ];
  showIndicator = true;

  switchIndicator(): void {
    this.showIndicator = !this.showIndicator;
  }
}

