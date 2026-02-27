# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dubverse is a Tauri 2 desktop application for video transcription, translation, and TTS dubbing. Currently at scaffold stage with a Vue 3 + TypeScript frontend and Rust backend.

## Build & Development Commands

```bash
# Start development (launches both Vite dev server and Tauri window)
npm run tauri dev

# Build for production
npm run tauri build

# Type-check and build frontend only
npm run build

# Generate app icons from a 1024x1024 PNG source
npx tauri icon path/to/source.png
```

## Architecture

```
src/                  # Vue 3 frontend (Composition API + <script setup>)
  main.ts             # App entry, mounts to #app
  App.vue             # Root component

src-tauri/            # Rust backend (Tauri 2)
  src/main.rs         # Entry point, calls dubverse_lib::run()
  src/lib.rs          # Tauri builder setup, command handlers
  tauri.conf.json     # App config (window size, bundle, CSP)
  Cargo.toml          # Rust dependencies
```

Frontend-backend communication uses Tauri's IPC via `@tauri-apps/api` (frontend) and `#[tauri::command]` (Rust).

## Tech Stack

- **Frontend:** Vue 3.5, TypeScript 5.6, Vite 6
- **Backend:** Rust (edition 2021), Tauri 2
- **Package manager:** npm
- **Rust lib name:** `dubverse_lib` (required on Windows to avoid cargo name conflicts)

## Key Config

- Vite dev server: port 1420, HMR port 1421
- Tauri dev URL: `http://localhost:1420`
- App identifier: `com.leica.dubverse`
- Window: 800x600, title "dubverse"
- Icons source SVG at `src-tauri/icons/icon.svg`
