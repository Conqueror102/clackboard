# Installing Clackboard

Download the file for your system from the [latest release](https://github.com/Conqueror102/clackboard/releases/latest).

> **Clackboard builds aren't signed by Apple or Microsoft yet**, so your system warns you the first time you open it. The steps below get past that. There are no automatic updates yet: download a new release to update.

## macOS (11 Big Sur or later, Apple Silicon or Intel)

1. Download `Clackboard_<version>_universal.dmg`, open it, and drag **Clackboard** into **Applications**.
2. Open Clackboard. macOS says it can't check the app: click **Done**, then go to **System Settings → Privacy & Security**, scroll down and click **Open Anyway**.
3. The panel opens under the icon in the menu bar. There's no Dock icon.

**Keyboard sounds** need **Input Monitoring**. The first time, macOS asks: allow it, or use the **Open Input Monitoring** button in the panel and switch on **Clackboard**. Sounds start by themselves. While you type a password, macOS hides your keys from every app, so Clackboard stays silent there.

**The keyboard lock** works best with the keyboard helper: press **Install helper** in the panel, enter your Mac password, then switch on **Clackboard Keyboard Helper** under Input Monitoring.

**Uninstall:** quit Clackboard from its panel, move it to the Bin, then remove the helper:

```bash
sudo "/Library/PrivilegedHelperTools/Clackboard Keyboard Helper.app/Contents/MacOS/clackboard-keyhelper" --keyhelper-uninstall
```

## Windows 10 and 11

1. Download `Clackboard_<version>_x64-setup.exe` and run it.
2. If Windows shows "Windows protected your PC", click **More info**, then **Run anyway**.
3. The installer sets Clackboard up for your user account only, with no administrator rights needed. On Windows 10 it may download Microsoft WebView2 if it's missing.
4. Clackboard appears in the system tray. If you don't see it, click the **^** arrow next to the clock; you can drag the icon onto the taskbar to keep it visible.

Sounds and the lock need no permission on Windows. While the keyboard is locked, **Ctrl + Alt + Del** always works, as Windows intends.

**Uninstall:** Settings → Apps → Installed apps → Clackboard → Uninstall.

## Linux (x86-64)

| File | Install |
| --- | --- |
| `Clackboard_<version>_amd64.deb` (Ubuntu, Debian, Mint, Pop!_OS) | `sudo apt install ./Clackboard_<version>_amd64.deb` |
| `Clackboard-<version>-1.x86_64.rpm` (Fedora, openSUSE) | `sudo dnf install ./Clackboard-<version>-1.x86_64.rpm` |
| `Clackboard_<version>_amd64.AppImage` (any distribution) | `chmod +x Clackboard_*.AppImage`, then run it |

Clackboard reads your keyboards directly (for sounds and the lock), so it works on X11 and Wayland. Add yourself to the `input` group once, then log out and back in:

```bash
sudo usermod -aG input $USER
```

> Members of the `input` group can read all keyboards and mice, so any program you run gets that access too. Only do this on a computer you trust.

- **GNOME:** tray icons need the [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/) (preinstalled on Ubuntu). Without it, use the Clackboard window.

**Uninstall:** `sudo apt remove clackboard` or `sudo dnf remove clackboard`, or delete the AppImage. Optionally remove yourself from the group: `sudo gpasswd -d $USER input`.

## Check a download

```bash
shasum -a 256 Clackboard_*        # macOS
sha256sum Clackboard_*            # Linux
```

```powershell
Get-FileHash .\Clackboard_*.exe   # Windows PowerShell
```

Compare with `SHA256SUMS.txt` on the release page.
