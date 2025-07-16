import { Component } from '@angular/core';
import {Router} from "@angular/router";
import {invoke} from "@tauri-apps/api/core";
import {AsyncPipe, NgIf} from "@angular/common";

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [
    NgIf,
    AsyncPipe
  ],
  templateUrl: './home.component.html',
  styleUrl: './home.component.css'
})
export class HomeComponent {
  constructor(private router: Router) {}

  resultText: string = 'checking...'

  adminStatus$!: Promise<{ success: true; value: boolean } | { success: false; error: string }>;

  ngOnInit(): void {
    this.adminStatus$ = this.checkAdmin(false); // call it once
  }
  async checkAdmin(showresult: boolean): Promise<{ success: true; value: boolean } | { success: false; error: string }> {
    try {
      const isAdmin: boolean = await invoke('check_admin');
      if(showresult) { this.resultText = isAdmin ? 'User is Admin' : 'User is NOT Admin'; }
      return { success: true, value: isAdmin };
    } catch (err) {
      if(showresult) { this.resultText = err as string; }
      return { success: false, error: err as string };
    }
  }

  goAdminPanel() {
    this.router.navigate(['/admin-console']).catch(console.error);
  }
  async testdbg(usern: string, showresult: boolean = true): Promise<{ success: true; value: boolean } | { success: false; error: string }> {
    try {
      await invoke('create_user', { username: usern, expiration: "01/01/1999" });
      if (showresult) { this.resultText = "success"; }
      return { success: true, value: true };
    } catch (err) {
      if(showresult) { this.resultText = err as string; }
      return { success: false, error: err as string };
    }
  }
}
