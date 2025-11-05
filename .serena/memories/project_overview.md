# LoLShorts Project Overview

## Purpose
LoLShorts is a League of Legends Auto-Recording & Editing Desktop Application that automatically captures gameplay highlights and provides tools for creating compilation videos.

## Core Features
- **Auto-Recording**: Circular buffer recording with 60-second replay window (6 × 10-second segments)
- **Event Detection**: Automatically detects kills, multi-kills, objectives (Baron, Dragon)
- **LCU Integration**: Connects to League Client Update API for real-time game state
- **Video Processing**: FFmpeg-based clip extraction and composition
- **License Tiers**: FREE (720p, watermark) vs PRO (1080p60, no watermark)
- **Cloud Sync**: Supabase integration for authentication, data storage, and cross-device sync

## Current Status
- **Phase 0 Complete** (20% overall): Core recording infrastructure, LCU integration, basic event detection
- **Phase 1-5 Planned**: Authentication, video processing, AI composition, editor UI, deployment

## Target Platform
- **Primary**: Windows 10/11 (64-bit)
- **Architecture**: Tauri 2.0 (Rust backend + React frontend)

## Key Technologies
- Backend: Rust, Tauri 2.0, Tokio async runtime
- Frontend: React 18, TypeScript, Zustand, shadcn/ui
- Video: FFmpeg CLI (bundled binary)
- Database: SQLite (local) + Supabase (cloud)
- Authentication: Supabase Auth with JWT tokens
- Monitoring: Sentry for error tracking

## Repository Location
C:\Users\wocks\RustroverProjects\LoLShorts

## Development Approach
- Test-Driven Development (TDD)
- Incremental implementation with continuous validation
- Wave-based development (5 waves planned)
- Evidence-based decisions (measure performance, validate assumptions)