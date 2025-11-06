# Audio Mixing Best Practices

**LoLShorts v1.2.0** - Create Professional-Sounding Auto-Edit Videos

---

## Overview

Audio mixing in LoLShorts combines **game audio** (in-game sounds, announcer, effects) with **background music** to create engaging, cinematic videos. Proper audio mixing can dramatically improve viewer retention and engagement.

### Why Audio Matters

**Statistics**:
- 60% of viewers watch YouTube Shorts with sound on
- Videos with music have 30% higher completion rates
- Poor audio mixing causes 40% of viewers to scroll away

**Audio affects**:
- **Emotional impact**: Music sets the tone (epic, chill, intense, funny)
- **Pacing**: Helps structure the video and maintain energy
- **Professionalism**: Balanced audio signals quality content
- **Watchability**: Prevents jarring volume spikes or muddy sound

---

## Audio Mixing Basics

### The Two Audio Channels

**Game Audio** (Primary):
- In-game sounds: Champion abilities, footsteps, auto-attacks
- Announcer: "Pentakill!", "An enemy has been slain"
- Voice chat: Team communication (if recorded)
- Ambient: Music, jungle camps, turrets

**Background Music** (Secondary):
- Royalty-free music track
- Sets mood and fills silence
- Loops to match video duration
- Mixed under game audio

### Volume Levels (0-100%)

**Game Audio Range**: 60-100%
- **100%**: Raw, unprocessed game audio (very loud)
- **70-80%**: Recommended default (clear but not overwhelming)
- **60%**: For music-heavy videos or commentary

**Background Music Range**: 0-40%
- **0%**: No music (gameplay-only audio)
- **20-30%**: Subtle background music (default)
- **40%**: Prominent music (chill montage style)

---

## Preset Audio Profiles

LoLShorts includes pre-configured audio mixing presets for common scenarios.

### Preset: Balanced (Default)

**Settings**:
- Game Audio: **70%**
- Background Music: **30%**

**Use Cases**:
- General highlights and montages
- Mixed gameplay types (kills + objectives)
- When unsure which preset to use

**Character**: Game sounds are clear, music provides subtle background energy.

**Example**: Typical pentakill highlight - you hear the announcer clearly with epic music underneath.

---

### Preset: Game Focus

**Settings**:
- Game Audio: **85%**
- Background Music: **15%**

**Use Cases**:
- Clutch plays requiring audio cues (hearing enemy footsteps)
- Announcer-heavy moments (pentakills, ace, baron steal)
- Tutorial/educational content where game sounds matter

**Character**: Game audio dominates, music barely audible as ambiance.

**Example**: Baron steal where you need to hear Smite timing and baron HP sounds.

---

### Preset: Music Focus

**Settings**:
- Game Audio: **50%**
- Background Music: **50%**

**Use Cases**:
- Chill montages with lo-fi/electronic music
- Stylized edits synced to music beats
- Videos where music drives the pace

**Character**: Balanced mix where music and game audio share equal prominence.

**Example**: Smooth Yasuo outplays set to chill electronic music.

---

### Preset: Pure Gameplay

**Settings**:
- Game Audio: **100%**
- Background Music: **0%**

**Use Cases**:
- Tournament footage or pro play
- VOD reviews and analysis
- Content where authenticity matters

**Character**: Raw, unaltered game audio with no music.

**Example**: Challenger-level gameplay showcase without any background music.

---

### Preset: Cinematic

**Settings**:
- Game Audio: **60%**
- Background Music: **40%**

**Use Cases**:
- Epic montages with dramatic music
- Story-driven compilations
- Emotional or hype-heavy content

**Character**: Music takes a prominent role, game audio supports.

**Example**: Season 14 highlights set to Two Steps From Hell - Victory.

---

## Finding the Right Balance

### Audio Mixing Decision Tree

```
What's the main focus of your video?

├─ Skill showcase / Clutch plays
│  └─ Use: Game Focus (85% / 15%)
│     Why: Need to hear ability timing, announcer, sound cues
│
├─ Relaxing montage / Flow gameplay
│  └─ Use: Music Focus (50% / 50%)
│     Why: Music sets the vibe, gameplay is secondary
│
├─ Hype/Epic moments / High energy
│  └─ Use: Cinematic (60% / 40%)
│     Why: Music amplifies excitement, game audio still clear
│
├─ Analysis / Tutorial / Educational
│  └─ Use: Pure Gameplay (100% / 0%)
│     Why: No distractions, authentic game experience
│
└─ Unsure / Mixed content
   └─ Use: Balanced (70% / 30%)
      Why: Safe default, works for most scenarios
```

### Testing Your Mix

**Before finalizing**:

1. **Export a test video** (30s sample)
2. **Watch on mobile** (80% of viewers use mobile devices)
3. **Check at different volumes**:
   - Low volume: Can you still hear announcer?
   - High volume: Does game audio overpower music?
4. **Ask yourself**:
   - Can I clearly hear important sounds (announcer, abilities)?
   - Does the music add to the experience or distract?
   - Would I watch this until the end?

**Red Flags**:
❌ Announcer is drowned out by music
❌ Music is barely audible (too quiet)
❌ Audio levels fluctuate wildly between clips
❌ Music and game sounds clash (muddy, hard to distinguish)

---

## Advanced Mixing Techniques

### Technique 1: Dynamic Mixing (Manual)

**Concept**: Adjust audio levels based on clip content.

**Example Workflow**:
1. Generate video with Balanced preset (70% / 30%)
2. Export to video editor (Premiere Pro, DaVinci Resolve)
3. Lower music during important moments:
   - Pentakill announcements: Reduce music to 10%
   - Clutch plays: Reduce music to 15%
   - Transition clips: Increase music to 50%
4. Re-export final video

**Result**: Professional dynamic mix that emphasizes key moments.

### Technique 2: Music Selection by BPM

**BPM** (Beats Per Minute) affects video pacing:

- **80-100 BPM**: Chill, relaxed, lo-fi (montages, smooth gameplay)
- **120-140 BPM**: Energetic, modern, EDM (hype plays, action)
- **140-160 BPM**: Intense, fast, aggressive (clutch plays, duels)
- **160+ BPM**: Frantic, drum & bass (pentakills, teamfights)

**Best Practice**: Match BPM to gameplay intensity.

**Example**:
- Slow-paced strategy game (Vel'Koz poke) → 90 BPM lo-fi
- Fast-paced duelist (Yasuo outplays) → 150 BPM EDM

### Technique 3: EQ and Compression (External Tools)

**Not supported in LoLShorts v1.2.0** - Use external audio editor for advanced control.

**EQ (Equalization)**:
- Boost game audio highs (2kHz-8kHz) for clarity
- Reduce music mids (500Hz-2kHz) to make room for game sounds
- Cut music lows (<100Hz) if game audio has bass

**Compression**:
- Apply light compression to game audio (-3dB threshold)
- Prevents sudden volume spikes (teamfights, flash sounds)
- Makes audio more consistent

**Tools**:
- **Audacity** (free): Basic EQ and compression
- **Adobe Audition** (paid): Professional audio editing
- **FL Studio** (paid): Music production with advanced mixing

---

## Royalty-Free Music Sources

### Best Free Music Libraries

**YouTube Audio Library** (youtube.com/audiolibrary)
- ✅ 100% free, no attribution required
- ✅ Extensive library (1000+ tracks)
- ✅ YouTube-safe (no copyright strikes)
- ⚠️ Music may be overused (less unique)

**Epidemic Sound** (epidemicsound.com)
- ✅ High-quality, professional music
- ✅ Extensive library, regular updates
- ✅ Licensed for commercial use
- ⚠️ Requires subscription ($15/month)

**Artlist** (artlist.io)
- ✅ Premium quality, cinematic music
- ✅ Unlimited downloads
- ✅ No attribution required
- ⚠️ Requires subscription ($25/month)

**Free Music Archive** (freemusicarchive.org)
- ✅ Completely free
- ✅ Wide variety of genres
- ⚠️ Check individual licenses (some require attribution)
- ⚠️ Quality varies

**NCS (NoCopyrightSounds)** (ncs.io)
- ✅ EDM and electronic music focus
- ✅ Free with attribution in description
- ✅ Popular in gaming community
- ⚠️ Recognizable tracks (less unique)

### Music Selection Guidelines

**DO**:
✅ Match music genre to content tone
✅ Choose instrumentals (no vocals, unless intentional)
✅ Verify license allows commercial use (if monetizing)
✅ Credit artist in video description

**DON'T**:
❌ Use copyrighted music (instant YouTube strikes)
❌ Use tracks with harsh vocals (distracts from gameplay)
❌ Use music with strong bass (clashes with game audio)
❌ Forget to check license terms

---

## Common Audio Mixing Mistakes

### Mistake 1: Music Too Loud

**Problem**: Background music drowns out game audio, can't hear announcer or key sounds.

**Symptoms**:
- Pentakill announcement is barely audible
- Ability sounds are muffled
- Viewers complain about audio balance

**Solution**:
✅ Reduce music to 20-30% (use Balanced or Game Focus preset)
✅ Test on mobile device at medium volume
✅ Compare to reference videos from successful creators

---

### Mistake 2: Music Too Quiet

**Problem**: Music is so quiet it doesn't add value, feels like an afterthought.

**Symptoms**:
- Music barely noticeable
- Video feels flat or boring
- No emotional impact from music

**Solution**:
✅ Increase music to 30-40% (use Balanced or Cinematic preset)
✅ Choose music with clear melody/beat
✅ Ensure music has dynamic range (not just ambient noise)

---

### Mistake 3: Volume Inconsistency Between Clips

**Problem**: Some clips are loud, others quiet, causing jarring transitions.

**Cause**: Different clips recorded at different times with varying game/system volumes.

**Solution**:
✅ LoLShorts automatically normalizes clip volumes (v1.2.0)
✅ If issues persist, manually adjust clip volumes in external editor
✅ Use audio compression to even out levels

---

### Mistake 4: Music Genre Mismatch

**Problem**: Music doesn't fit the gameplay style or content tone.

**Examples**:
- Intense dubstep for slow, strategic gameplay (mismatch)
- Sad piano music for hype pentakill moment (mismatch)
- Relaxed lo-fi for fast-paced duelist highlights (mismatch)

**Solution**:
✅ Match music energy to gameplay intensity
✅ Consider your target audience's music preferences
✅ Test different genres and get feedback

---

### Mistake 5: Using Copyrighted Music

**Problem**: YouTube copyright strikes, video muted or taken down.

**Consequences**:
- Video muted (audio removed entirely)
- Revenue goes to copyright holder (if monetized)
- Channel strikes (3 strikes = channel termination)
- Cannot dispute if music is genuinely copyrighted

**Solution**:
✅ Only use royalty-free music or music you have rights to
✅ Verify license before using
✅ Keep license documentation
✅ Use YouTube Audio Library for guaranteed safety

---

## Audio Mixing Workflows

### Workflow 1: Quick Auto-Edit (5 minutes)

**Goal**: Fast video generation with default audio

1. Select games and duration
2. Click "Audio" tab
3. Choose **Balanced** preset (70% / 30%)
4. Upload royalty-free music from YouTube Audio Library
5. Generate video
6. Done!

**Result**: Good quality, no manual adjustments needed.

---

### Workflow 2: Refined Mix (15 minutes)

**Goal**: Optimized audio for specific content type

1. Select games and duration
2. Click "Audio" tab
3. Determine content focus:
   - Skill showcase? → **Game Focus** (85% / 15%)
   - Montage? → **Music Focus** (50% / 50%)
   - Epic moments? → **Cinematic** (60% / 40%)
4. Upload music matching gameplay BPM
5. Generate test video (30s sample)
6. Adjust if needed:
   - Too loud? Lower music by 10%
   - Too quiet? Increase music by 10%
7. Generate full video

**Result**: Polished audio mix tailored to content.

---

### Workflow 3: Professional Mix (30+ minutes)

**Goal**: Broadcast-quality audio with external editing

1. Generate video with **Pure Gameplay** preset (100% / 0%)
2. Export raw video with game audio only
3. Import to audio editor (Audacity, Adobe Audition)
4. Apply processing:
   - Normalize game audio to -3dB
   - Add light compression (3:1 ratio)
   - EQ: Boost highs (5kHz) for clarity
5. Import background music track
6. Manually adjust music levels per section:
   - Lower during announcer moments
   - Raise during quiet gameplay
7. Export mixed audio
8. Import to video editor (Premiere Pro)
9. Replace audio track
10. Export final video

**Result**: Professional-grade audio mix with dynamic levels.

---

## Audio Mixing Checklist

Before finalizing your video, verify:

### Pre-Generation
- [ ] Music file is royalty-free or licensed
- [ ] Music file format is supported (MP3, WAV, M4A)
- [ ] Audio preset matches content type
- [ ] Music BPM matches gameplay intensity

### Post-Generation
- [ ] Announcer is clearly audible
- [ ] Music adds value (not distracting)
- [ ] No volume spikes or clipping
- [ ] Audio sounds good on mobile device
- [ ] Music loops smoothly (no abrupt cuts)

### Before Upload
- [ ] Music credit in video description
- [ ] License terms allow platform (YouTube/TikTok)
- [ ] Audio passes YouTube copyright check
- [ ] Final audio quality is acceptable

---

## Troubleshooting Audio Issues

### "Audio is clipping / distorting"

**Cause**: Combined game + music audio exceeds 0dB (maximum volume).

**Solution**:
✅ Reduce both game and music volumes by 10-20%
✅ Use: Game 60%, Music 25% (total 85%)
✅ LoLShorts automatically prevents clipping, but this may occur in some edge cases

---

### "Music doesn't loop smoothly"

**Cause**: Music track has intro/outro that doesn't loop seamlessly.

**Solution**:
✅ Use tracks designed for looping (check music source)
✅ Edit music in Audacity to remove intro/outro
✅ Find music with "loopable" or "seamless" tags

---

### "Game audio too quiet in specific clips"

**Cause**: Some source videos recorded with low game volume.

**Solution**:
✅ Increase game audio to 85-90%
✅ If issue persists, edit specific clips in external editor
✅ Normalize clip audio before importing to LoLShorts

---

### "Background music is too repetitive"

**Cause**: Music track is short and loops many times.

**Solution**:
✅ Use longer music tracks (3-5 minutes minimum)
✅ Choose music with variations (verse, chorus, bridge)
✅ Avoid simple 15-30 second loops

---

## FAQ

### Q: Can I use multiple music tracks in one video?
**A**: Not in v1.2.0. One background music track per video. For multiple tracks, edit in external video editor.

### Q: What if I want no music at all?
**A**: Use the **Pure Gameplay** preset (100% game / 0% music). Do not upload a music file.

### Q: Can I adjust left/right audio balance (stereo)?
**A**: Not in v1.2.0. Audio is mixed to stereo center. Planned for v1.3.0.

### Q: Does LoLShorts support 5.1 surround sound?
**A**: No, output is stereo AAC 192kbps. Sufficient for YouTube Shorts / TikTok.

### Q: How do I avoid audio sync issues?
**A**: LoLShorts automatically syncs audio and video. If issues occur, report as a bug.

### Q: Can I preview audio mix before generating?
**A**: Not in v1.2.0. Generate a test video (30s) to check mix. Planned for v1.3.0.

---

## Next Steps

1. **Try Balanced preset**: Generate your first video with default audio (70% / 30%)
2. **Experiment with presets**: Test different presets to find your style
3. **Build music library**: Download 10-15 royalty-free tracks for future videos
4. **Refine your mix**: Adjust levels based on viewer feedback
5. **Go advanced**: Learn external audio editing for professional results

**Resources**:
- [YouTube Audio Library](https://youtube.com/audiolibrary)
- [Audacity Tutorial](https://manual.audacityteam.org/)
- [Audio Mixing Fundamentals](https://www.youtube.com/watch?v=TEjOdqZFvhY)

---

**Version**: 1.2.0
**Last Updated**: 2025-01-06
**Next**: [Troubleshooting Guide →](./TROUBLESHOOTING.md)
