import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TrainingEnvComponent } from './training-env.component';

describe('TrainingEnvComponent', () => {
  let component: TrainingEnvComponent;
  let fixture: ComponentFixture<TrainingEnvComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TrainingEnvComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(TrainingEnvComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
