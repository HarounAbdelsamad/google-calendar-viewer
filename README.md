<p align="center">
  <img src="app-icon.svg" width="128" height="128" alt="Calendar Viewer logo">
</p>

<h1 align="center">Calendar Viewer</h1>

<p align="center">
  A tiny, fast, <b>read-only</b> Google Calendar app for the Windows desktop.<br>
  Built with Tauri 2 (Rust) + React.
</p>

---

## Features

- **Month, Week, Day and Agenda views** of all your Google calendars in one window
- **Calendar toggles** in the sidebar, in each calendar's own Google color; your choices are remembered
- **Event details** on click: time, location, description, plus **Join Google Meet** and **Open in Google Calendar** buttons
- **Custom background picture** from your PC, shown behind see-through panels; event text stays fully opaque
- **Always up to date**: refreshes every 5 minutes, when the window regains focus, or on demand with ↻
- **Stays signed in** between launches; sign out at any time from the sidebar
- **Light and dark mode** that follows Windows, Monday-first weeks, 24-hour times

## Why it's good

| | |
| --- | --- |
| **Read-only by design** | It only asks Google for the `calendar.readonly` permission, so it *cannot* create, change or delete anything. |
| **Tiny and light** | A ~2 MB installer, using roughly 50 MB of RAM. It uses the WebView2 engine that ships with Windows instead of bundling a whole browser like Electron (150 MB+). |
| **Private** | No servers and no tracking. The app talks directly to Google from your PC. |
| **Secure token handling** | Sign-in uses the OAuth PKCE flow in your normal browser. The refresh token is stored in **Windows Credential Manager**, and the UI never sees any token. |
| **Fast** | A Rust backend fetches all calendars in parallel. |

## Setup: Google credentials (one time, ~5 min)

The app needs its own OAuth client from Google Cloud. It's free.

1. **Create a project.** Go to <https://console.cloud.google.com/>, open the project dropdown, click **New Project**, name it `Calendar Viewer` and click **Create**. Then select it.
2. **Enable the API.** Go to **APIs & Services → Library**, search **Google Calendar API** and click **Enable**.
3. **Set up the consent screen.** Go to **APIs & Services → OAuth consent screen** and click **Get started**.
   - App name `Calendar Viewer`, your email as the support email, audience **External**, your email as the contact. Click **Create**.
   - **Data Access** → **Add or remove scopes** → tick `.../auth/calendar.readonly` → **Update** → **Save**.
   - **Audience** → **Test users** → **Add users** → add your own Gmail address. *Without this, sign-in shows "Access blocked".*
4. **Create the client.** Go to **Clients → Create client**, choose application type **Desktop app**, then click **Create**. Copy the **Client ID** and **Client secret**. The new console doesn't show the secret again.
5. **Add them to the project.** Copy `.env.example` to `.env` in the project root and fill it in:

   ```
   GOOGLE_CLIENT_ID=1234567890-abc.apps.googleusercontent.com
   GOOGLE_CLIENT_SECRET=GOCSPX-xxxxxxxxxxxx
   ```

   `.env` is git-ignored. The values are built into the binary at compile time (see `src-tauri/build.rs`), so **rebuild after changing them**.

> **Testing vs. published:** while the consent screen is in *Testing*, Google expires sign-ins after 7 days and only listed test users can sign in. Go to **Audience → Publish app** to remove the 7-day limit. You will still see the "Google hasn't verified this app" warning; click **Continue**.

## Build

### Prerequisites (Windows)

- [Node.js](https://nodejs.org/) 20 or newer
- [Rust](https://rustup.rs/) stable (1.85 or newer): `rustup update stable`
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload
- WebView2 is already included in Windows 10 and 11

### Run in development

```bash
npm install
npm run tauri dev
```

### Build the installer

```bash
npm run tauri build
```

The installers are written to `src-tauri/target/release/bundle/`:

- `nsis/Calendar Viewer_<version>_x64-setup.exe`: per-user install, no admin needed, creates a desktop shortcut
- `msi/Calendar Viewer_<version>_x64_en-US.msi`

### Change the icon

Edit `app-icon.svg`, then regenerate all icon sizes:

```bash
npx tauri icon app-icon.svg
```

## Usage

1. Click **Sign in with Google**. Your browser opens; pick your account and allow read-only access. You can close the tab once it says *Signed in*.
2. Use the sidebar checkboxes to show or hide calendars.
3. Under **Background**, click **Choose picture…** to set a wallpaper behind the calendar, or **Remove** to go back to the plain look.
4. Click any event to see its details.

## Troubleshooting

| Message | Fix |
| --- | --- |
| *Access blocked: app has not completed verification* | Add your Gmail address under **Audience → Test users**. |
| *invalid_client* / *Missing GOOGLE_CLIENT_ID* | Check `.env`, then rebuild. |
| *Your session expired* | Normal every 7 days in Testing mode; sign in again, or publish the app. |
| *Google is rate-limiting requests* | Wait a minute, then press ↻. |

## Project structure

| Path | Purpose |
| --- | --- |
| `src-tauri/src/auth.rs` | OAuth PKCE sign-in, token refresh, Credential Manager storage |
| `src-tauri/src/google.rs` | Calendar API v3 calls: calendar list and events (parallel, paginated) |
| `src-tauri/src/background.rs` | Stores the chosen background picture in the app data folder |
| `src-tauri/src/lib.rs` | Tauri commands exposed to the UI |
| `src/App.tsx` | Sign-in screen, layout, background state |
| `src/components/CalendarView.tsx` | FullCalendar v7 month, week, day and agenda views |
| `src/components/Sidebar.tsx` | Calendar toggles, background picker, sign out |
| `src/components/EventDetails.tsx` | Read-only event popup |

## Tech stack

[Tauri 2](https://tauri.app/) · Rust (`reqwest`, `tokio`, `keyring`) · React 19 + TypeScript · [FullCalendar 7](https://fullcalendar.io/) · Vite
