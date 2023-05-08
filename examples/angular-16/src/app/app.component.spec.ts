import { ComponentFixture, TestBed, waitForAsync } from '@angular/core/testing';
import { AppComponent } from './app.component';
import { AppModule } from './app.module';

xdescribe('App: Ng2Bootstrap', () => {
  let fixture: ComponentFixture<AppComponent>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let context: any;

  beforeEach(waitForAsync(() => {
    TestBed.configureTestingModule({
      imports: [AppModule]
    })
      .compileComponents()
      .then(() => {
        fixture = TestBed.createComponent(AppComponent);
        context = fixture.componentInstance;
      });
  }));

  it('should create the app', (() => {
      expect(context).toBeTruthy();
    })
  );
});
