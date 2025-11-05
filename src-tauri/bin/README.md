# FFmpeg Binaries Directory

This directory contains FFmpeg binaries that are bundled with the LoLShorts installer.

## Required Files

- `ffmpeg.exe` - Video processing and encoding
- `ffprobe.exe` - Video analysis and metadata extraction

## How to Obtain

### Automated (Recommended)
Run the preparation script:
```powershell
cd ../build_scripts
.\prepare_ffmpeg.ps1
```

### Manual Download
1. Download FFmpeg from: https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip
2. Extract `ffmpeg.exe` and `ffprobe.exe` from the `bin/` folder in the archive
3. Place both files in this directory

## Verification

After obtaining the binaries, verify they work:
```bash
.\ffmpeg.exe -version
.\ffprobe.exe -version
```

Expected file sizes:
- ffmpeg.exe: ~70-80 MB
- ffprobe.exe: ~70-75 MB

## Note

These binaries are NOT committed to git due to their large size (~150MB total).
They must be prepared locally before building the production installer.
