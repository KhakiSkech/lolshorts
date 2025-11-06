# Canvas Design Tutorial

**LoLShorts v1.2.0** - Create Custom Overlays for Your Auto-Edit Videos

---

## What is a Canvas Overlay?

A **canvas overlay** is a graphical layer that appears on top of your gameplay video. It can include:

- **Text elements**: Channel name, video title, captions, timestamps
- **Image elements**: Logos, watermarks, champion icons, sponsor graphics
- **Background layers**: Solid colors, gradients, or background images

Canvas overlays help you:
✅ Brand your content with consistent visual identity
✅ Add context and storytelling elements
✅ Create professional-looking videos
✅ Stand out on social media feeds

---

## Canvas Basics

### Canvas Dimensions

**Resolution**: 360x640 pixels (9:16 aspect ratio scaled)
- Matches YouTube Shorts / TikTok / Instagram Reels format
- Scales to 1080x1920 during video rendering
- Position coordinates are **percentage-based** (0-100)

### Coordinate System

```
(0, 0) ────────────────────► X (100)
  │
  │    Your Canvas Area
  │    360x640 pixels
  │
  ▼ Y (100)
```

**Position Examples**:
- Top-left corner: `x: 0, y: 0`
- Top-center: `x: 50, y: 0`
- Center: `x: 50, y: 50`
- Bottom-right: `x: 100, y: 100`

### Element Layers

Elements are rendered in the order they're added:
1. **Background** (bottom layer)
2. **Images** (middle layer, order by addition)
3. **Text** (top layer, order by addition)

**Tip**: Add important text last to ensure it appears on top.

---

## Step-by-Step: Your First Canvas

### Step 1: Open Canvas Editor

1. Go to **Auto-Edit** page
2. Click **"Canvas"** tab
3. You'll see a blank 360x640 preview canvas

### Step 2: Add Background

**Option A: Solid Color**
```
1. Click "Background" → "Color"
2. Enter hex color (e.g., #1a1a2e for dark blue)
3. Preview updates immediately
```

**Option B: Gradient**
```
1. Click "Background" → "Gradient"
2. Enter two hex colors separated by colon (e.g., #1a1a2e:#16213e)
3. Creates top-to-bottom gradient
```

**Option C: Image**
```
1. Click "Background" → "Image"
2. Select image file (PNG, JPG recommended)
3. Image stretches to fill 360x640 canvas
```

**Best Practice**: Use dark backgrounds (black, dark blue, dark purple) for better text readability.

### Step 3: Add Channel Name

```
1. Click "Add Text"
2. Enter your channel name (e.g., "ProPlayerGG")
3. Set properties:
   - Font: "Arial Bold" (or custom font)
   - Size: 36 pixels
   - Color: #FFFFFF (white)
   - Outline: #000000 (black, optional)
4. Position: x: 50, y: 5 (top-center)
5. Click "Apply"
```

**Result**: White text with black outline at top of screen.

### Step 4: Add Event Caption (Optional)

```
1. Click "Add Text"
2. Enter caption (e.g., "PENTAKILL")
3. Set properties:
   - Font: "Impact"
   - Size: 48 pixels
   - Color: #FFD700 (gold)
   - Outline: #000000 (black)
4. Position: x: 50, y: 90 (bottom-center)
5. Click "Apply"
```

### Step 5: Add Logo Watermark (Optional)

```
1. Click "Add Image"
2. Select your logo (PNG with transparency recommended)
3. Set size: 60x60 pixels
4. Position: x: 90, y: 5 (top-right corner)
5. Click "Apply"
```

### Step 6: Save Template

```
1. Click "Save Template"
2. Enter template name: "My Brand Template"
3. Template saved for future use
```

---

## Canvas Design Patterns

### Pattern 1: Minimal Branding

**Goal**: Simple, professional look without cluttering gameplay

**Elements**:
- Background: None (transparent)
- Text: Channel name (top-center, 32px, white with black outline)
- Image: Small logo (60x60, bottom-right corner, 20% opacity)

**Use Cases**: Gameplay-focused content, minimalist style

```yaml
Template: Minimal Brand
Background: Transparent
Elements:
  - Text: "YourChannelName"
    Position: x: 50, y: 3
    Size: 32px
    Color: #FFFFFF
    Outline: #000000
  - Image: "logo.png"
    Position: x: 85, y: 92
    Size: 60x60
    Opacity: 20%
```

### Pattern 2: Bold Title Card

**Goal**: Eye-catching intro/outro with large text

**Elements**:
- Background: Gradient (dark purple to black)
- Text 1: Video title (center, 56px, bold, gold)
- Text 2: Subtitle (below title, 28px, white)
- Image: Champion splash art (background, 50% opacity)

**Use Cases**: Video intros, episode titles, dramatic effect

```yaml
Template: Bold Title
Background: Gradient (#2d1b69:#000000)
Elements:
  - Text: "TOP 5 PENTAKILLS"
    Position: x: 50, y: 40
    Size: 56px
    Color: #FFD700
    Outline: #000000
  - Text: "Week 42 Highlights"
    Position: x: 50, y: 55
    Size: 28px
    Color: #FFFFFF
```

### Pattern 3: Streamer Overlay

**Goal**: Consistent branding with social media info

**Elements**:
- Background: Semi-transparent gradient bar (bottom)
- Text 1: Channel name (top-left, 28px)
- Text 2: Social handle (bottom-center, 24px)
- Image 1: Logo (top-right corner)
- Image 2: Rank emblem (bottom-left corner)

**Use Cases**: Stream highlights, consistent branding across videos

```yaml
Template: Streamer Brand
Background: Gradient overlay bar
Elements:
  - Image: "logo.png"
    Position: x: 90, y: 5
    Size: 70x70
  - Text: "ProPlayerGG"
    Position: x: 10, y: 5
    Size: 28px
    Color: #FFFFFF
  - Text: "@ProPlayerGG"
    Position: x: 50, y: 95
    Size: 24px
    Color: #FFCC00
  - Image: "rank-challenger.png"
    Position: x: 10, y: 90
    Size: 50x50
```

---

## Advanced Techniques

### Custom Fonts

**Adding Custom Fonts**:
1. Download TrueType font (.ttf) or OpenType font (.otf)
2. Place in `C:\Users\[You]\AppData\Local\LoLShorts\fonts\`
3. Restart LoLShorts
4. Font appears in Canvas Editor font dropdown

**Recommended Free Fonts**:
- **Impact**: Bold, readable, great for captions
- **Bebas Neue**: Modern, tall, perfect for titles
- **Montserrat**: Clean sans-serif, professional
- **Roboto**: Google's default, always safe
- **Oswald**: Condensed, space-efficient

**Font Pairing Tips**:
- Title: Bold font (Impact, Bebas Neue)
- Body text: Clean sans-serif (Roboto, Montserrat)
- Don't mix more than 2 font families

### Text Outlines & Shadows

**Outline** (recommended for readability):
```yaml
Text: "PENTAKILL"
Color: #FFD700 (gold)
Outline: #000000 (black)
Outline Width: 2px (automatic)
```

**Effect**: Black border around text ensures visibility on any background.

**Shadow** (not yet supported in v1.2.0):
- Planned for v1.3.0
- Will add depth and professionalism

### Image Transparency & Opacity

**Transparent PNGs**:
- Use PNG files with alpha channel for logos
- Allows gameplay to show through
- Perfect for watermarks

**Opacity Control**:
- Not yet supported in v1.2.0
- Workaround: Pre-edit images in Photoshop/GIMP to desired opacity
- Planned for v1.3.0

### Responsive Positioning

**Percentage-Based Positioning**:
- Always use percentages (0-100) instead of pixels
- Ensures canvas scales correctly to 1080x1920
- Example: `x: 50, y: 5` always means "top-center" regardless of resolution

**Safe Areas**:
- Top/bottom **5%**: Safe for text and logos
- Left/right **10%**: Avoid placing critical elements (may be cropped on some devices)
- Center **80%**: Primary gameplay area, avoid heavy overlays

---

## Design Best Practices

### Readability

**DO**:
✅ Use high contrast (white text on dark background)
✅ Add text outlines (2-3px black outline on all text)
✅ Keep text large (minimum 24px for body, 36px for titles)
✅ Use simple, bold fonts (Impact, Bebas Neue, Montserrat)

**DON'T**:
❌ Use thin fonts (<400 weight)
❌ Place text over busy backgrounds without outline
❌ Use script/cursive fonts (hard to read on mobile)
❌ Overlay text on important gameplay areas

### Visual Hierarchy

**Size Matters**:
- **Title**: 48-64px (most important)
- **Channel name**: 28-36px (secondary)
- **Captions**: 24-32px (supporting info)
- **Social handles**: 20-24px (least important)

**Positioning Importance**:
- **Top-center**: Primary branding (channel name)
- **Center**: Key messages (event captions, titles)
- **Bottom-center**: Social media handles
- **Corners**: Logos, watermarks (subtle)

### Branding Consistency

**Color Palette**:
- Choose 2-3 brand colors and stick to them
- Example palette: Gold (#FFD700), Dark Blue (#1a1a2e), White (#FFFFFF)
- Use gold for highlights, dark blue for backgrounds, white for text

**Template Library**:
- Create 3-5 templates for different content types
  - "Standard Gameplay" (minimal overlay)
  - "Montage Intro" (bold title card)
  - "Stream Highlight" (full branding)
- Reuse templates for consistent look across videos

### Mobile Optimization

**Remember**:
- 80% of YouTube Shorts viewers watch on mobile
- Text must be readable on 5-6 inch screens
- Keep important elements in safe areas

**Test**:
- Preview generated video on your phone
- Check if text is readable from arm's length
- Adjust sizes if needed (usually increase by 20-30%)

---

## Canvas Templates Library

### Template: Clean Gameplay
```yaml
Name: "Clean Gameplay"
Background: Transparent
Elements:
  - Text: "[Your Channel]"
    Position: {x: 50, y: 4}
    Font: "Montserrat Bold"
    Size: 32px
    Color: #FFFFFF
    Outline: #000000
  - Image: "logo_small.png"
    Position: {x: 88, y: 4}
    Size: 50x50
Use Case: Default for all videos, minimal branding
```

### Template: Hype Montage
```yaml
Name: "Hype Montage"
Background: Gradient (#FF0844:#FFB199)
Elements:
  - Text: "EPIC MOMENTS"
    Position: {x: 50, y: 45}
    Font: "Impact"
    Size: 64px
    Color: #FFFFFF
    Outline: #000000
  - Text: "Subscribe for More!"
    Position: {x: 50, y: 60}
    Font: "Roboto"
    Size: 28px
    Color: #FFFFFF
Use Case: Intro card for montages
```

### Template: Tutorial Style
```yaml
Name: "Tutorial"
Background: Color (#1a1a2e)
Elements:
  - Text: "HOW TO: [Technique]"
    Position: {x: 50, y: 8}
    Font: "Bebas Neue"
    Size: 40px
    Color: #FFD700
    Outline: #000000
  - Text: "Step 1: [Description]"
    Position: {x: 50, y: 88}
    Font: "Roboto"
    Size: 24px
    Color: #FFFFFF
Use Case: Educational content, tips & tricks
```

---

## Troubleshooting Canvas Issues

### "Canvas elements not appearing in final video"

**Causes**:
- Elements positioned outside canvas bounds (x/y > 100 or < 0)
- Text color same as background color (invisible)
- Image file path incorrect or file deleted

**Solutions**:
✅ Check element positions in Canvas Editor preview
✅ Test with high contrast colors first (white text on black background)
✅ Verify image files exist in specified paths

### "Text is too small / unreadable on mobile"

**Cause**: Font size too small for mobile viewing

**Solution**:
✅ Increase text size by 50% (if 24px, try 36px)
✅ Add black outline to all text (improves readability dramatically)
✅ Test on actual mobile device

### "Logo/watermark looks pixelated"

**Cause**: Image resolution too low for 1080x1920 output

**Solution**:
✅ Use high-resolution images (minimum 300x300 for 60x60 canvas size)
✅ Export logos as PNG with transparency
✅ Use vector graphics (SVG) when possible (convert to PNG at high resolution)

### "Canvas preview looks good but final video doesn't match"

**Cause**: Preview canvas is 360x640, but final video is 1080x1920 (3x scale)

**Solution**:
✅ This is expected behavior - preview is 1:3 scale
✅ Test with actual video generation to see true quality
✅ Use percentage-based positioning (already default)

---

## Canvas Editor Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + N` | Create new canvas template |
| `Ctrl + S` | Save current template |
| `Ctrl + Z` | Undo last change |
| `Ctrl + Y` | Redo change |
| `Delete` | Remove selected element |
| `Arrow Keys` | Nudge selected element (1% increments) |
| `Shift + Arrow` | Nudge selected element (10% increments) |

---

## Next Steps

1. **Create your first template**: Start with "Clean Gameplay" pattern
2. **Test with a video**: Generate a 60s auto-edit with your template
3. **Refine on mobile**: Check readability on your phone
4. **Build template library**: Create 2-3 templates for different content types
5. **Iterate**: Adjust based on viewer feedback and engagement

**Examples & Inspiration**:
- [LoLShorts Template Gallery](#) (coming soon)
- [Community Templates Discord](#) (coming soon)

---

**Version**: 1.2.0
**Last Updated**: 2025-01-06
**Next**: [Audio Mixing Best Practices →](./AUDIO_MIXING.md)
