'use strict';

const { app, BrowserWindow, dialog, Menu, shell } = require('electron');
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

app.setName('twatch');

let mainWindow = null;
let daemon = null;
let daemonLog = null;
let daemonOwned = false;
let started = false;
let quitting = false;
let opened = false;

function daemonBinary() {
  if (process.env.TWATCHD_BIN) return process.env.TWATCHD_BIN;
  if (app.isPackaged) {
    return path.join(
      process.resourcesPath,
      process.platform === 'win32' ? 'twatchd.exe' : 'twatchd',
    );
  }
  return process.platform === 'win32' ? 'twatchd.exe' : 'twatchd';
}

function daemonLogPath() {
  try {
    return path.join(app.getPath('logs'), 'twatchd.log');
  } catch {
    return null;
  }
}

function openDaemonLog() {
  try {
    const logPath = daemonLogPath();
    if (!logPath) return null;
    fs.mkdirSync(path.dirname(logPath), { recursive: true });
    return fs.createWriteStream(logPath, { flags: 'a' });
  } catch {
    return null;
  }
}

const CSD_DESKTOPS = new Set([
  'gnome',
  'gnome-classic',
  'gnome-flashback',
  'ubuntu',
  'unity',
  'pantheon',
  'budgie',
  'budgie-desktop',
  'niri',
]);

const SSD_DESKTOPS = new Set([
  'kde',
  'plasma',
  'xfce',
  'lxqt',
  'lxde',
  'mate',
  'x-cinnamon',
  'cinnamon',
  'i3',
  'sway',
  'hyprland',
  'bspwm',
  'dwm',
  'awesome',
  'qtile',
  'regolith',
]);

function desktopTokens() {
  return [process.env.XDG_CURRENT_DESKTOP, process.env.XDG_SESSION_DESKTOP, process.env.DESKTOP_SESSION]
    .filter(Boolean)
    .join(':')
    .split(':')
    .map(token => token.trim().toLowerCase())
    .filter(Boolean);
}

function shouldUseFramelessWindow() {
  if (process.platform !== 'linux') return false;
  if (process.env.NIRI_SOCKET) return true;
  if (String(process.env.XDG_SESSION_TYPE || '').toLowerCase() !== 'wayland') return false;
  const tokens = desktopTokens();
  if (tokens.some(token => SSD_DESKTOPS.has(token))) return false;
  return tokens.some(token => CSD_DESKTOPS.has(token));
}

function stopDaemon() {
  quitting = true;
  if (daemon && daemonOwned && !daemon.killed) {
    daemon.kill('SIGTERM');
  }
}

function fail(message) {
  if (quitting) return;
  quitting = true;
  const logPath = daemonLogPath();
  const detail = logPath ? `${message}\n\nDaemon log: ${logPath}` : String(message);
  console.error(`twatch-electron: ${detail}`);
  dialog.showErrorBox('twatch could not start', detail);
  app.quit();
}

function createWindow(url) {
  if (mainWindow) {
    mainWindow.loadURL(url);
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.focus();
    return;
  }

  const frameless = shouldUseFramelessWindow();
  const options = {
    width: 1280,
    height: 820,
    minWidth: 900,
    minHeight: 600,
    show: false,
    backgroundColor: '#131722',
    icon: path.join(__dirname, 'icon.png'),
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  };

  if (frameless && process.platform === 'linux') {
    options.titleBarStyle = 'hidden';
  }

  const win = new BrowserWindow(options);
  mainWindow = win;
  opened = true;
  win.setMenuBarVisibility(false);
  win.webContents.setWindowOpenHandler(({ url: external }) => {
    void shell.openExternal(external);
    return { action: 'deny' };
  });
  win.webContents.on('did-fail-load', (_event, code, description, failedUrl) => {
    if (failedUrl.startsWith('http://127.0.0.1')) {
      fail(`Could not load the local console: ${description} (${code})`);
    }
  });
  win.once('ready-to-show', () => win.show());
  win.on('closed', () => {
    mainWindow = null;
  });
  win.loadURL(url);
}

function handleDaemonLine(line) {
  const match = line.match(/https?:\/\/[^\s]+/);
  if (!match) return;
  if (line.includes('twatchd listening on')) {
    createWindow(match[0]);
    return;
  }
  if (line.includes('twatchd is already running at')) {
    daemonOwned = false;
    createWindow(match[0]);
  }
}

function startDaemon() {
  if (started) return;
  started = true;

  const binary = daemonBinary();
  if (app.isPackaged) {
    if (!fs.existsSync(binary)) {
      fail(`The bundled twatchd daemon is missing at ${binary}`);
      return;
    }
    if (process.platform !== 'win32') {
      try {
        fs.chmodSync(binary, 0o755);
      } catch {
        // The packaged file may already be executable.
      }
    }
  }

  daemonLog = openDaemonLog();
  daemonLog?.write(`\n[twatch-electron] starting ${binary}\n`);

  daemon = spawn(binary, ['--port', '0'], {
    env: process.env,
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  });
  daemonOwned = true;

  let buffer = '';
  daemon.stdout.setEncoding('utf8');
  daemon.stdout.on('data', chunk => {
    daemonLog?.write(chunk);
    buffer += chunk;
    let newline;
    while ((newline = buffer.indexOf('\n')) >= 0) {
      const line = buffer.slice(0, newline).trim();
      buffer = buffer.slice(newline + 1);
      if (line) handleDaemonLine(line);
    }
  });
  daemon.stderr.setEncoding('utf8');
  daemon.stderr.on('data', chunk => {
    daemonLog?.write(chunk);
    console.error(chunk.trimEnd());
  });
  daemon.on('error', error => fail(`Could not start twatchd: ${error.message}`));
  daemon.on('exit', (code, signal) => {
    daemon = null;
    daemonLog?.write(`[twatch-electron] twatchd exited (${signal || code})\n`);
    if (!opened) {
      fail(`twatchd exited before opening the console (${signal || code})`);
    }
  });
}

const gotLock = app.requestSingleInstanceLock();
if (!gotLock) {
  app.quit();
} else {
  app.on('second-instance', () => {
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.show();
      mainWindow.focus();
    }
  });

  app.whenReady().then(() => {
    if (process.platform === 'win32') app.setAppUserModelId('io.github.risotto.twatch');
    Menu.setApplicationMenu(null);
    startDaemon();
  });

  app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
  });

  app.on('before-quit', stopDaemon);
}
