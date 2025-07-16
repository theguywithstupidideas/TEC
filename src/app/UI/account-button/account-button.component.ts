import {Component, Input, OnInit} from '@angular/core';
import {invoke} from "@tauri-apps/api/core";
import {confirm} from "@tauri-apps/plugin-dialog";

@Component({
  selector: 'app-account-button',
  standalone: true,
  imports: [],
  templateUrl: './account-button.component.html',
  styleUrl: './account-button.component.css'
})
export class AccountButtonComponent implements OnInit {
  @Input() user: string | null = null;
  username: string = "";

  ngOnInit(): void {
    if (this.user) {
      this.username = this.user;
    }
  }
  async DeleteUser(user: string): Promise<void> {
    try {
      await invoke('delete_user', {username: user});
      await confirm(
          'Account cancellato.',
          {title: 'TEC', kind: 'info'}
      )
    }
    catch (error) {
      await confirm(
          `${error}`,
          {title: 'TEC - Errore', kind: 'error'}
      )
    }
  }
}
