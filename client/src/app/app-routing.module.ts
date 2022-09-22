import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { TilemapComponent } from './tilemap/tilemap.component';
import { WorldComponent } from './world/world.component';

const routes: Routes = [
  { path: 'tilemap', component: TilemapComponent },
  { path: 'world', component: WorldComponent },
  { path: '', pathMatch: 'full', redirectTo: 'tilemap' },
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule],
})
export class AppRoutingModule {}
