'use strict'

const fs = require('node:fs')
const path = require('node:path')

const target = path.join(__dirname, '..', 'resources')
fs.mkdirSync(target, { recursive: true })

function copy(source, name) {
  if (!source || !fs.existsSync(source)) {
    throw new Error(`Missing media tool: ${source}`)
  }
  fs.copyFileSync(source, path.join(target, name))
  fs.chmodSync(path.join(target, name), 0o755)
}

copy(require('ffmpeg-static'), process.platform === 'win32' ? 'ffmpeg.exe' : 'ffmpeg')
copy(require('ffprobe-static').path, process.platform === 'win32' ? 'ffprobe.exe' : 'ffprobe')
console.log(`Prepared ffmpeg/ffprobe in ${target}`)
