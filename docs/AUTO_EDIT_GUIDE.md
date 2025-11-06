# Auto-Edit Feature User Guide

**LoLShorts v1.2.0** - Automatic Video Generation from Gameplay Clips

---

## Overview

The Auto-Edit feature automatically creates short-form videos (60s, 120s, or 180s) by intelligently selecting and combining your best gameplay moments. Perfect for sharing highlights on YouTube Shorts, TikTok, or Instagram Reels.

### What Auto-Edit Does

1. **Selects** the most exciting clips from your games based on event priority
2. **Combines** clips into a seamless 9:16 vertical video
3. **Applies** custom canvas overlays with text and graphics (optional)
4. **Mixes** background music with game audio (optional)
5. **Exports** a ready-to-upload YouTube Short

---

## Quick Start (3 Steps)

### Step 1: Select Games

1. Open **Auto-Edit** from the sidebar
2. Select at least **1 game** (2-3 recommended for 60s videos)
3. Games with more clips provide better content selection

**Tip**: Games with pentakills, baron steals, or clutch plays will produce the best highlights.

### Step 2: Choose Duration

Pick your target video length:
- **60 seconds** - Perfect for quick highlights (1-2 games)
- **120 seconds** - Extended montage (2-3 games)
- **180 seconds** - Full compilation (3-5 games)

**Best Practice**: Start with 60s for your first auto-edit to see results quickly.

### Step 3: Generate

1. Click **"Generate Video"**
2. Wait 30-90 seconds (processing time varies by clip count)
3. Find your video in the output folder when complete

**Output Location**: `C:\Users\[You]\Videos\LoLShorts\auto_edit\`

---

## Advanced Customization

### Canvas Overlays (Optional)

Add visual flair with custom text, graphics, and backgrounds.

**When to Use**:
- Branding your content (add channel name/logo)
- Creating themed videos (tournament highlights, challenge runs)
- Adding context (event names, dates, captions)

**See**: [Canvas Design Tutorial](./CANVAS_TUTORIAL.md) for detailed guide.

### Background Music (Optional)

Mix royalty-free music with game audio for cinematic effect.

**When to Use**:
- Muting repetitive in-game sounds
- Creating emotional tone (epic, chill, intense)
- Matching music to gameplay pace

**See**: [Audio Mixing Best Practices](./AUDIO_MIXING.md) for detailed guide.

---

## How Auto-Edit Selects Clips

Auto-Edit uses a **priority scoring system** to choose the best moments:

### Priority Levels (1-5)

| Priority | Event Type | Example |
|----------|------------|---------|
| **5** | Pentakill | 5 kills in 10 seconds |
| **4** | Quadrakill, Baron Steal | 4 kills, objective steal |
| **3** | Multikill, Dragon, Baron | Double/Triple kill, objective secure |
| **2** | Assist Streak, Turret Destroy | 3+ assists, first turret |
| **1** | Single Kill, Vision | Individual kill, ward placement |

### Selection Algorithm

1. **Filter by threshold**: Only considers clips with priority ≥ threshold (default: 2)
2. **Sort by priority**: Highest priority clips first
3. **Fill target duration**: Selects clips until reaching target length
4. **Trim excess**: Crops clips if total duration exceeds target

**Example**:
- Target: 60s video
- Available: 3x pentakill (30s each), 2x baron (20s each)
- Selection: 2x pentakill (60s total) - highest priority clips

---

## Generation Process (Behind the Scenes)

Understanding the stages helps troubleshoot issues and set expectations.

### Stage 1: Selecting Clips (10%)
- Queries database for clips from selected games
- Applies priority filtering and sorting
- Calculates optimal clip selection for target duration
- **Time**: <5 seconds

### Stage 2: Preparing Clips (30%)
- Extracts clips from source video files using FFmpeg
- Trims clips to configured durations
- Scales/crops to 9:16 aspect ratio (1080x1920)
- **Time**: 10-30 seconds (depends on clip count)

### Stage 3: Concatenating (50%)
- Merges all clips into single video
- Re-encodes to consistent format (H.264, AAC)
- Applies compression (CRF 23)
- **Time**: 15-45 seconds (depends on total duration)

### Stage 4: Applying Canvas (70%)
*(Only if canvas template is enabled)*
- Overlays custom graphics and text
- Renders final composition
- **Time**: 5-15 seconds

### Stage 5: Mixing Audio (90%)
*(Only if background music is enabled)*
- Balances game audio and music levels
- Loops music to match video duration
- Applies audio normalization
- **Time**: 5-10 seconds

### Complete (100%)
- Video saved to output folder
- Metadata stored in database
- Ready for upload

---

## Performance Expectations

### Processing Times

| Target Duration | Clip Count | Expected Time |
|----------------|------------|---------------|
| 60s | 3-5 clips | **20-30 seconds** |
| 120s | 6-10 clips | **40-60 seconds** |
| 180s | 9-15 clips | **60-90 seconds** |

**Performance Target**: <30 seconds per minute of output video

**Factors Affecting Speed**:
- CPU performance (single-threaded FFmpeg operations)
- Source video resolution (1080p faster than 4K)
- Canvas overlay complexity (simple text faster than images)
- Audio mixing (music processing adds overhead)

### File Sizes

| Duration | File Size (Typical) | Bitrate |
|----------|---------------------|---------|
| 60s | 15-25 MB | ~2-3 Mbps |
| 120s | 30-50 MB | ~2-3 Mbps |
| 180s | 45-75 MB | ~2-3 Mbps |

**Compression Settings**: CRF 23 (good quality), H.264 codec, AAC audio 192k

---

## Common Workflows

### Workflow 1: Quick Daily Highlight
**Goal**: Create today's best play in 60 seconds

1. Select today's game with most kills
2. Choose 60s duration
3. Click Generate (no customization needed)
4. Upload to YouTube Shorts immediately

**Time**: 2 minutes total (30s generation + 90s upload)

### Workflow 2: Weekly Montage
**Goal**: Compile week's best moments into 2-minute video

1. Select 3-4 best games from the week
2. Choose 120s duration
3. Add canvas overlay with week number ("Week 42 Highlights")
4. Add epic background music (e.g., Imagine Dragons)
5. Generate and review
6. Upload with hashtags #leagueoflegends #shorts

**Time**: 5-10 minutes total (including customization)

### Workflow 3: Champion Mastery Showcase
**Goal**: Showcase your champion proficiency

1. Filter games to specific champion (e.g., Yasuo)
2. Select 5+ games with high KDA
3. Choose 180s duration
4. Add canvas with champion name and mastery level
5. Add intense electronic music
6. Generate compilation
7. Upload with champion-specific hashtags

**Time**: 15 minutes total (including game selection and customization)

---

## Tips & Best Practices

### Selecting Games

**✅ Good Practices**:
- Choose games with diverse event types (kills, objectives, teamfights)
- Mix early/mid/late game clips for variety
- Select games where you played different roles/champions
- Prioritize recent games (fresher gameplay)

**❌ Avoid**:
- Games with only 1-2 clips (insufficient content)
- Games with similar repetitive plays
- Old games (outdated meta/patches)
- Games where you performed poorly (low priority clips)

### Target Duration Selection

**60 seconds**:
- Best for single epic play or quick montage
- Ideal for testing Auto-Edit first time
- High engagement rate on social media
- Requires 1-2 well-recorded games

**120 seconds**:
- Balanced length for storytelling
- Good for multi-game compilations
- Still maintains viewer attention
- Requires 2-3 games with clips

**180 seconds**:
- Extended showcase or tutorial format
- Best for skilled players with consistent performance
- May lose viewer attention if not well-paced
- Requires 3-5+ games with diverse content

### Canvas Overlay Design

**Keep It Simple**:
- Minimal text (channel name, video title)
- High contrast colors (white text on dark background)
- Small logos in corners (don't cover gameplay)
- Avoid busy backgrounds (distracts from video)

**Branding Elements**:
- Channel name (top or bottom center)
- Social media handle (small, corner placement)
- Logo watermark (semi-transparent, 10% opacity)

### Audio Mixing

**Default Settings** (Good for most videos):
- Game audio: 70%
- Background music: 30%

**Adjust Based on Content**:
- **Pentakill/clutch plays**: 80% game / 20% music (emphasize announcer)
- **Chill montage**: 50% game / 50% music (balanced vibe)
- **Tutorial/commentary**: 90% game / 10% music (focus on sounds)

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + G` | Open game selection |
| `Ctrl + Enter` | Start generation (if ready) |
| `Ctrl + R` | Retry failed generation |
| `Space` | Play/pause preview (after generation) |
| `Ctrl + O` | Open output folder |
| `Ctrl + N` | Start new auto-edit (reset) |

---

## Troubleshooting

For detailed troubleshooting, see [Troubleshooting Guide](./TROUBLESHOOTING.md).

### Quick Fixes

**"No clips found"**
- ✅ Solution: Record more games with the recording feature enabled

**"Not enough clips to create 60s video"**
- ✅ Solution: Select more games OR reduce target duration

**"FFmpeg not found"**
- ✅ Solution: Install FFmpeg from [ffmpeg.org](https://ffmpeg.org/download.html)

**Generation takes too long (>2 minutes)**
- ✅ Solution: Close other applications, reduce clip count, or check CPU usage

---

## FAQ

### Q: How many games should I select?
**A**: For 60s videos, select 1-2 games with high activity. For 120s, select 2-3 games. For 180s, select 3-5 games.

### Q: Can I manually choose which clips to include?
**A**: Not in v1.2.0. The algorithm automatically selects based on priority. Manual clip selection is planned for v1.3.0.

### Q: What's the best aspect ratio for YouTube Shorts?
**A**: Auto-Edit exports 9:16 (1080x1920) which is optimized for YouTube Shorts, TikTok, and Instagram Reels.

### Q: Can I edit the generated video further?
**A**: Yes! The output MP4 can be edited in any video editor (Premiere Pro, DaVinci Resolve, CapCut).

### Q: Does Auto-Edit work offline?
**A**: Yes, once games are recorded. However, you need an internet connection for initial setup and updates.

### Q: Can I use copyrighted music?
**A**: No! Only use royalty-free music or music you have rights to. YouTube will flag copyrighted content.

### Q: How do I add my channel intro/outro?
**A**: Use canvas overlays for branding. For full intro/outro videos, edit the Auto-Edit output in a video editor.

---

## Next Steps

1. **Try your first auto-edit**: Select 1 game, 60s duration, click Generate
2. **Experiment with canvas**: Add your channel name and simple text overlay
3. **Add background music**: Use royalty-free music from YouTube Audio Library
4. **Upload to YouTube Shorts**: Test engagement with your first short
5. **Iterate**: Adjust canvas/audio based on viewer feedback

**Need Help?**
- Join our Discord: [discord.gg/lolshorts](https://discord.gg/lolshorts)
- Report issues: [GitHub Issues](https://github.com/yourusername/lolshorts/issues)
- Watch tutorials: [YouTube Channel](https://youtube.com/@lolshorts)

---

**Version**: 1.2.0
**Last Updated**: 2025-01-06
**Next**: [Canvas Design Tutorial →](./CANVAS_TUTORIAL.md)
