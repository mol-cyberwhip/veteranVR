# Rookie Desktop - Design Specification

## Overview

This document defines the visual design language, layout architecture, and interaction patterns for the Rookie Desktop application — a cross-platform (macOS-first) Quest VR sideloading tool built with Tauri.

The design draws inspiration from:
- **SideQuest** (direct competitor — dark theme, game card grid, category pills)
- **Heroic Games Launcher** (open-source game launcher — sidebar nav, library grid, detail pages)
- **GOG Galaxy** (polished dark UI — unified library, view toggles, clean typography)
- **Steam Library** (sidebar collections, detail panels, status badges)
- Modern Dribbble game launcher concepts (see `design_inspiration/` screenshots)

---

## Design Principles

1. **Dark-first, VR-native** — Dark backgrounds reduce eye strain and feel native to the VR/gaming space
2. **Information density without clutter** — Show what matters (game name, size, status, version) without overwhelming
3. **Device connection is king** — The headset connection status is always visible and prominent
4. **Progressive disclosure** — Simple browsing by default, power features accessible but not in the way
5. **macOS-native feel** — Respect platform conventions (traffic lights, system fonts, vibrancy)

---

## Color System

### Core Palette

```
Background Layer 0 (App shell):     #0f1114    (near-black)
Background Layer 1 (Sidebar):       #161921    (dark navy-black)
Background Layer 2 (Content):       #1a1d24    (dark charcoal)
Background Layer 3 (Cards/Panels):  #22262f    (elevated surface)
Background Layer 4 (Hover/Active):  #2a2f3a    (lighter surface)

Border/Dividers:                    #2e3340    (subtle separation)
Border Focus:                       #4a90d9    (blue focus ring)
```

### Text Hierarchy

```
Text Primary:      #f0f2f5    (headings, game names, primary content)
Text Secondary:    #a0a8b4    (descriptions, metadata, labels)
Text Muted:        #6b7280    (timestamps, tertiary info)
Text Disabled:     #4a4f5a    (inactive elements)
```

### Accent Colors

```
Brand Primary:     #4a90d9    (buttons, links, active nav items)
Brand Hover:       #5ba0e9    (button hover state)
Brand Active:      #3a7dc5    (button pressed)

Success:           #34d399    (installed, connected, complete)
Warning:           #fbbf24    (update available, low storage)
Error:             #f87171    (failed, disconnected, error)
Info:              #60a5fa    (informational badges)

Favorite:          #f472b6    (heart icon, favorite badge)
New/Fresh:         #a78bfa    (newly added titles)
```

### Status-Specific Colors

```
Device Connected:      #34d399 (green dot + text)
Device Unauthorized:   #fbbf24 (amber dot + prompt)
Device Disconnected:   #6b7280 (grey dot + text)
Device Error:          #f87171 (red dot + message)

Game Not Installed:    (no badge — default state)
Game Installed:        #34d399 (green checkmark badge)
Game Update Available: #4a90d9 (blue arrow-up badge)
Game Downloading:      #4a90d9 (blue progress ring)
Game Installing:       #fbbf24 (amber spinner)
Game Failed:           #f87171 (red warning badge)
```

---

## Typography

```
Font Stack:         -apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI", Roboto, sans-serif
Mono Stack:         "SF Mono", "Fira Code", "Cascadia Code", Menlo, monospace

Heading XL:         24px / 700 weight / -0.02em tracking    (page titles)
Heading L:          18px / 600 weight / -0.01em tracking    (section headers)
Heading M:          15px / 600 weight / 0em tracking        (card titles, game names)
Body:               14px / 400 weight / 0em tracking        (descriptions, metadata)
Body Small:         12px / 400 weight / 0.01em tracking     (labels, badges, timestamps)
Caption:            11px / 500 weight / 0.02em tracking     (tiny labels, version numbers)
```

---

## Spacing & Layout

```
Base unit:          4px
Common spacings:    4, 8, 12, 16, 20, 24, 32, 40, 48, 64

Border radius:
  - Small:          6px   (badges, chips, small buttons)
  - Medium:         10px  (cards, inputs, buttons)
  - Large:          14px  (panels, modals)
  - Round:          999px (pills, avatars, status dots)

Shadows:
  - Card:           0 2px 8px rgba(0, 0, 0, 0.3)
  - Elevated:       0 8px 24px rgba(0, 0, 0, 0.4)
  - Modal:          0 16px 48px rgba(0, 0, 0, 0.5)
```

---

## Application Shell Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│  [Traffic Lights]  ·····  Rookie Desktop  ·····  [Device Status]   │  ← Title Bar (36px)
├──────────┬──────────────────────────────────────────────────────────┤
│          │  [🔍 Search games...]    [Sort ▾]  [≡ List] [⊞ Grid]   │  ← Toolbar (48px)
│  NAV     ├──────────────────────────────────────────────────────────┤
│  SIDEBAR │                                                          │
│          │                                                          │
│  Library │              MAIN CONTENT AREA                           │
│  --------│              (Game Grid / List)                          │
│  Downlds │                                                          │
│  Install │                                                          │
│  --------│                                                          │
│  Device  │                                                          │
│  --------│                                                          │
│  Settings│                                                          │
│          │──────────────────────────────────────────────────────────│
│          │  [Download Queue / Progress Bar]                         │  ← Status Bar (40px)
└──────────┴──────────────────────────────────────────────────────────┘
   220px                        remainder (flex)
```

### Title Bar (36px height)

- **macOS**: Use native title bar with traffic light buttons at top-left. Title centered.
- **Right side**: Device status indicator — always visible.
  - Format: `● Connected: Quest 3 (72% 🔋)` or `○ No device connected`
  - Clicking opens the Device panel in the sidebar

### Navigation Sidebar (220px width, fixed)

- Background: Layer 1 (`#161921`)
- Sections separated by thin `#2e3340` dividers
- Nav items: 40px tall, 12px left padding, 10px border-radius
- Active item: `#4a90d9` left border (3px), `#1e2a3a` background
- Hover: `#22262f` background

```
Nav Items:
─────────────────
  📚  Library          ← Game catalog (default view)
  ⬇️  Downloads        ← Active/queued downloads
  📦  Installed        ← Installed apps on device
─────────────────
  📱  Device           ← Device info, storage, battery
  🔌  Wireless ADB     ← Connect wirelessly
─────────────────
  ⭐  Favorites        ← Favorited games
  🆕  New Releases     ← Recently added
─────────────────
  ⚙️  Settings         ← App preferences
  📋  Logs             ← Diagnostics
```

### Toolbar (48px height)

- Background: Layer 2 (`#1a1d24`)
- Search input: Full width minus controls, `#22262f` background, `#2e3340` border
  - Placeholder: "Search 2500+ games..."
  - Debounced (200ms), results filter in real-time
- Sort dropdown: `Name ▾`, `Date ▾`, `Size ▾`, `Popularity ▾`
- View toggle: Two icon buttons (list / grid), active one highlighted with `#4a90d9`

### Status Bar (40px height)

- Background: Layer 1 (`#161921`)
- Shows current download progress when active:
  `Downloading: Game Name — 45% · 12.3 MB/s · 2:30 remaining · [████████░░░░░░] · [Pause] [Cancel]`
- When idle: `Ready · 127 games available · 24.3 GB free on Quest`
- Click to expand full download queue panel

---

## Key Screens & User Journeys

---

### Screen 1: Library — Grid View (Default Home)

The primary screen. Shows game catalog as a responsive grid of cards.

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ ░░░░░░░░ │  │ ░░░░░░░░ │  │ ░░░░░░░░ │  │ ░░░░░░░░ │         │
│  │ ░ THUMB ░ │  │ ░ THUMB ░ │  │ ░ THUMB ░ │  │ ░ THUMB ░ │        │
│  │ ░░░░░░░░ │  │ ░░░░░░░░ │  │ ░░░░░░░░ │  │ ░░░░░░░░ │         │
│  │          │  │   ✓ Inst │  │   ⬆ Upd  │  │   ★ Fav  │         │
│  │ Game Nam  │  │ Game Nam  │  │ Game Nam  │  │ Game Nam  │        │
│  │ 1.2 GB   │  │ 3.4 GB   │  │ 850 MB   │  │ 2.1 GB   │         │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘         │
│                                                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ ...      │  │ ...      │  │ ...      │  │ ...      │         │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘         │
│                                                                    │
└──────────────────────────────────────────────────────────────────┘
```

**Game Card (Grid View)**:
- Dimensions: ~200px wide, auto height (responsive grid `repeat(auto-fill, minmax(190px, 1fr))`)
- Thumbnail: 16:9 aspect ratio, rounded top corners (10px), with fallback placeholder
- Status badge: Top-right corner overlay (pill shape, 6px radius)
  - Green `✓ Installed` / Blue `⬆ Update` / None for not-installed
- Favorite star: Top-left corner, only shown on hover unless favorited
- Title: 15px semibold, max 2 lines, ellipsis overflow
- Metadata row: size in muted text, version number in caption
- Hover: Slight scale (1.02), elevated shadow, border glow `rgba(74, 144, 217, 0.3)`
- Click: Opens Game Detail panel (right side or overlay)

**Filter Chips** (above grid, horizontally scrollable):
```
[All] [Installed] [Updates Available] [Favorites] [New This Week]
```
- Active chip: `#4a90d9` background, white text
- Inactive: `#22262f` background, `#a0a8b4` text, `#2e3340` border

---

### Screen 2: Library — List View

Alternative compact view for information-dense browsing.

```
┌──────────────────────────────────────────────────────────────────┐
│  Name              │ Package        │ Version │ Size  │ Status   │
├──────────────────────────────────────────────────────────────────┤
│  ⊞ Beat Saber      │ com.beatga...  │ 1.37.0  │ 2.1G  │ ✓ Inst  │
│  ⊞ Blade & Sorcery │ com.warp...    │ 1.2.0   │ 3.4G  │ ⬆ Upd   │
│  ⊞ Gorilla Tag     │ com.anoth...   │ 1.1.85  │ 850M  │ —        │
│  ⊞ Population One  │ com.bigbo...   │ 3.2.1   │ 4.1G  │ ✓ Inst  │
│  ⊞ Superhot VR     │ com.super...   │ 1.0.22  │ 1.8G  │ ✓ Inst  │
│  ...               │                │         │       │          │
└──────────────────────────────────────────────────────────────────┘
```

- Row height: 44px
- Alternating row backgrounds: `#1a1d24` / `#1e2128`
- Hover: `#22262f` background
- Selected: `#1e2a3a` background with left `#4a90d9` border
- Sortable columns: Click header to sort, arrow indicator for direction
- Status column: Colored badge pills (same colors as grid)
- Tiny thumbnail (32x32) at left edge of each row

---

### Screen 3: Game Detail Panel

Opens when clicking a game. Can be a right-side slide-in panel (40% width) or a modal overlay.

```
┌─────────────────────────────────────────┐
│  ← Back                    [★ Fav] [⋮] │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │                                 │    │
│  │        GAME THUMBNAIL           │    │
│  │         (16:9 hero)             │    │
│  │                                 │    │
│  └─────────────────────────────────┘    │
│                                         │
│  Beat Saber                             │
│  com.beatgames.beatsaber                │
│                                         │
│  Version: 1.37.0    Size: 2.1 GB       │
│  Updated: 2024-12-15                    │
│  Downloads: 145,230                     │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │  ⬇ Download & Install          │    │  ← Primary action button
│  └─────────────────────────────────┘    │     (full-width, 48px tall,
│                                         │      #4a90d9 bg, white text)
│  ┌──────────┐ ┌────────────────────┐    │
│  │ 🗑 Delete │ │ 📂 Open Folder    │    │  ← Secondary actions
│  └──────────┘ └────────────────────┘    │     (outlined buttons)
│                                         │
│  ── Notes ──────────────────────────    │
│  Rich text game notes/description.      │
│  May contain install instructions,      │
│  compatibility warnings, etc.           │
│                                         │
│  ── Install Info ───────────────────    │
│  Type: APK + OBB                        │
│  Has install.txt: Yes                   │
│  Package: com.beatgames.beatsaber       │
│                                         │
└─────────────────────────────────────────┘
```

**Primary Action Button States**:
- Not downloaded: `⬇ Download & Install` (blue `#4a90d9`)
- Downloading: Progress bar fills button, `45% — 12 MB/s` (blue gradient)
- Downloaded (not installed): `📦 Install to Quest` (green `#34d399`)
- Installed: `✓ Installed — v1.37.0` (green outline, muted)
- Update available: `⬆ Update to v1.38.0` (blue `#4a90d9`)
- No device: `⬇ Download Only` (blue, with tooltip explaining no device)
- Error: `⚠ Retry Install` (red `#f87171`)

---

### Screen 4: Downloads & Queue

Shows active download, queue, and history.

```
┌──────────────────────────────────────────────────────────────────┐
│  DOWNLOADS                                         [Clear Done]  │
│                                                                    │
│  ── Active ─────────────────────────────────────────────────────  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  ⊞ Beat Saber                                              │  │
│  │  [████████████████░░░░░░░░░░░] 67%  ·  18.2 MB/s           │  │
│  │  1.41 GB / 2.1 GB  ·  ETA: 0:38    [⏸ Pause] [✕ Cancel]  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  ── Queued ─────────────────────────────────────────────────────  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  ≡  Blade & Sorcery          3.4 GB        [↑] [↓] [✕]   │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │  ≡  Gorilla Tag               850 MB       [↑] [↓] [✕]   │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │  ≡  Population One           4.1 GB        [↑] [↓] [✕]   │  │
│  └────────────────────────────────────────────────────────────┘  │
│  Drag ≡ handles to reorder queue                                  │
│                                                                    │
│  ── Completed ──────────────────────────────────────────────────  │
│  │  ✓  Superhot VR              1.8 GB     Installed 2m ago  │  │
│  │  ✓  Pistol Whip              1.2 GB     Installed 15m ago │  │
│                                                                    │
└──────────────────────────────────────────────────────────────────┘
```

**Progress Bar Design**:
- Track: `#22262f` background, 8px tall, 4px radius
- Fill: Linear gradient `#4a90d9` → `#5ba0e9`, animated shimmer
- When paused: Fill becomes `#6b7280` (grey)
- When failed: Fill becomes `#f87171` (red), shows retry button

**Queue Items**:
- 60px row height
- Drag handle `≡` on left (6 dots pattern)
- Reorder buttons `↑` `↓` and remove `✕` on right
- Hover shows full row background highlight

---

### Screen 5: Device Panel

Shows connected device information, storage, and controls.

```
┌──────────────────────────────────────────────────────────────────┐
│  DEVICE                                                           │
│                                                                    │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  ● Connected via USB                                       │  │
│  │                                                            │  │
│  │  Meta Quest 3                                              │  │
│  │  Serial: 1WMHH8######                                     │  │
│  │                                                            │  │
│  │  🔋 Battery: 72%   [████████████████░░░░░░]                │  │
│  │                                                            │  │
│  │  💾 Storage                                                │  │
│  │  [████████████████████░░░░░░░░░░] 68.2 GB / 128 GB        │  │
│  │  Used: 68.2 GB  ·  Free: 59.8 GB                         │  │
│  │                                                            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  ── Connected Devices ──────────────────────────────────────────  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  ◉  1WMHH8######   Quest 3   USB   72%  ← selected       │  │
│  │  ○  192.168.1.42   Quest 2   WiFi  45%                    │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  ── Wireless ADB ───────────────────────────────────────────────  │
│  IP Address: [192.168.1.___]  Port: [5555]  [Connect]            │
│  ☐ Remember and auto-reconnect                                   │
│                                                                    │
│  ── Quick Actions ──────────────────────────────────────────────  │
│  [📸 Pull Screenshots]  [🎥 Pull Recordings]  [🔧 ADB Console]  │
│                                                                    │
└──────────────────────────────────────────────────────────────────┘
```

**Device Status Banner** (appears at top of content area when relevant):
- **No device**: Amber banner across content top — `"No Quest connected. Plug in via USB or connect wirelessly. Download-only mode active."`
- **Unauthorized**: Red banner — `"Quest connected but unauthorized. Please put on headset and accept the USB debugging prompt."`
- **Connected**: Brief green flash notification, then status in title bar

**Storage Bar**:
- Track: `#22262f`, 12px tall, 6px radius
- Fill: Gradient based on usage:
  - < 70%: `#34d399` (green)
  - 70-90%: `#fbbf24` (amber)
  - > 90%: `#f87171` (red)

---

### Screen 6: Installed Apps

Shows what's installed on the connected device.

```
┌──────────────────────────────────────────────────────────────────┐
│  INSTALLED APPS (47)                              [Refresh]       │
│                                                                    │
│  🔍 Filter installed apps...                                      │
│                                                                    │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ ⊞  Beat Saber           v1.37.0   2.1 GB                  │  │
│  │     com.beatgames.beatsaber                                │  │
│  │     [⬆ Update Available: v1.38.0]  [Backup] [Uninstall]  │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ ⊞  Superhot VR          v1.0.22   1.8 GB                  │  │
│  │     com.superhot.superhot                                  │  │
│  │     [✓ Latest]  [Backup] [Uninstall]                      │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ ⊞  Gorilla Tag          v1.1.85   850 MB                  │  │
│  │     com.anothertag.gorillatag                              │  │
│  │     [✓ Latest]  [Extract APK] [Backup] [Uninstall]        │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  Total installed size: 42.7 GB                                    │
│                                                                    │
└──────────────────────────────────────────────────────────────────┘
```

**Uninstall Confirmation Dialog**:
```
┌─────────────────────────────────────┐
│  Uninstall Beat Saber?              │
│                                     │
│  ☐ Also remove OBB data             │
│  ☐ Also remove save data            │
│                                     │
│  ⚠ Save data will be lost unless    │
│    you create a backup first.       │
│                                     │
│  [Backup First]  [Cancel] [Remove]  │
└─────────────────────────────────────┘
```

---

### Screen 7: Settings

```
┌──────────────────────────────────────────────────────────────────┐
│  SETTINGS                                                         │
│                                                                    │
│  ── Downloads ──────────────────────────────────────────────────  │
│  Download location:   [/Users/me/Downloads/Rookie  ] [Browse]    │
│  Max concurrent:      [1 ▾]                                       │
│  Bandwidth limit:     [☐ Unlimited] [_____ MB/s]                 │
│  Auto-install after:  [● Yes ○ No]                               │
│  Keep archives:       [● Yes ○ No]                               │
│                                                                    │
│  ── Device ─────────────────────────────────────────────────────  │
│  Auto-connect WiFi:   [● Yes ○ No]                               │
│  Saved endpoint:      [192.168.1.42:5555]                        │
│  Keep-awake on ops:   [● Yes ○ No]                               │
│                                                                    │
│  ── Display ────────────────────────────────────────────────────  │
│  Default view:        [● Grid ○ List]                            │
│  Default sort:        [Name ▾]                                    │
│  Show popularity:     [● Yes ○ No]                               │
│                                                                    │
│  ── Advanced ───────────────────────────────────────────────────  │
│  Offline mode:        [○ Yes ● No]                               │
│  Proxy:               [_________________________]                 │
│  Diagnostic ID:       abc123...  [Reset] [Copy]                  │
│  Clean temp files:    [Clean Now]                                 │
│                                                                    │
│  ── About ──────────────────────────────────────────────────────  │
│  Version: 1.0.0                                                   │
│  [Check for Updates]  [Export Logs]  [View Changelog]            │
│                                                                    │
└──────────────────────────────────────────────────────────────────┘
```

---

### Screen 8: Install Progress Timeline

Shown when an install operation is in progress (modal overlay or panel).

```
┌─────────────────────────────────────────┐
│  Installing: Beat Saber                  │
│                                         │
│  ✓  Downloaded archive         2.1 GB  │
│  ✓  Extracted to release dir           │
│  ✓  Cleaned hash directory             │
│  ●  Installing APK...          ◌◌◌     │ ← spinning indicator
│  ○  Pushing OBB files                   │
│  ○  Running install.txt                 │
│  ○  Verifying installation              │
│                                         │
│  Step 4 of 7  ·  Elapsed: 1:23          │
│                                         │
│  [Cancel Install]                       │
└─────────────────────────────────────────┘
```

**Step States**:
- `✓` Completed: `#34d399` green checkmark
- `●` In Progress: `#4a90d9` blue filled dot with spinner
- `○` Pending: `#6b7280` grey hollow dot
- `✗` Failed: `#f87171` red X with error message beneath

---

## Interaction Patterns

### Hover States
- Cards: 1.02x scale, elevated shadow, subtle blue border glow
- Buttons: Background lightens by 10%, cursor pointer
- Nav items: `#22262f` background fill
- Table rows: Full-row `#22262f` highlight

### Loading States
- Skeleton screens for cards while catalog loads (pulsing `#22262f` → `#2a2f3a`)
- Spinner (16px, `#4a90d9`) next to text for inline operations
- Full-screen centered spinner with "Loading catalog..." for first launch

### Toast Notifications
- Position: Bottom-right, 16px from edges
- Width: 360px max
- Background: `#22262f` with colored left border (4px)
- Auto-dismiss: 5s for info/success, 10s for warnings, persistent for errors
- Stack: Max 3 visible, older ones slide down

### Empty States
- Centered illustration + text
- "No games match your search" with [Clear Search] button
- "No device connected" with [Connect Guide] button
- "Download queue is empty" with "Browse the library to get started"

### Keyboard Shortcuts
```
⌘F          Focus search
⌘1-9        Switch sidebar sections
⌘⇧G        Toggle grid/list view
⌘R          Refresh catalog/device
Space       Toggle select in list view
Enter       Open selected game detail
Escape      Close detail panel / clear search
⌘,          Open settings
```

---

## Responsive Behavior

The Tauri window is resizable. Key breakpoints:

| Window Width | Layout Changes |
|---|---|
| > 1200px | Full layout: sidebar + content + detail panel side-by-side |
| 900-1200px | Sidebar + content; detail opens as overlay |
| 700-900px | Collapsible sidebar (icon-only mode, 56px); content fills |
| < 700px | Bottom tab nav replaces sidebar; single column |

### Minimum Window Size
- **Width**: 680px
- **Height**: 480px

---

## Animation & Motion

- **Transitions**: 150ms ease-out for most state changes
- **Page transitions**: 200ms slide-in from right (detail panel)
- **Progress bars**: Smooth width transitions, subtle shimmer animation
- **Skeleton loading**: 1.5s pulse animation
- **Toast entrance**: 300ms slide-up + fade-in
- **Card hover**: 200ms scale + shadow transition

---

## Iconography

Use a consistent icon set throughout. Recommended: **Lucide Icons** (open-source, clean, consistent weight).

Key icons needed:
- Navigation: `library`, `download`, `package`, `smartphone`, `wifi`, `star`, `sparkles`, `settings`, `file-text`
- Actions: `play`, `pause`, `x`, `refresh-cw`, `trash-2`, `folder-open`, `upload`, `chevron-up`, `chevron-down`
- Status: `check-circle`, `alert-circle`, `info`, `arrow-up-circle`, `loader`
- Device: `battery`, `hard-drive`, `usb`, `wifi`

Icon size: 18px for nav, 16px for inline, 14px for badges.

---

## Reference Screenshots

The `design_inspiration/` directory contains captured reference screenshots:

| File | Description |
|---|---|
| `sidequest_home.png` | SideQuest homepage — dark theme, hero layout |
| `sidequest_apps_catalog.png` | SideQuest app grid — card design, category pills, sort/filter bar |
| `heroic_homepage.png` | Heroic Games Launcher — sidebar nav, game grid, dark theme |
| `dribbble_game_launcher_grid.png` | Dribbble inspiration — game launcher sidebar + overview patterns |
| `dribbble_game_launcher_grid2.png` | Dribbble inspiration — additional game launcher concepts |
| `dribbble_game_library.png` | Dribbble inspiration — game library desktop app concepts |

---

## Component Reference

### Button Variants

| Variant | Background | Text | Border | Use |
|---|---|---|---|---|
| Primary | `#4a90d9` | `#ffffff` | none | Main actions (Download, Install) |
| Success | `#34d399` | `#0f1114` | none | Confirmations (Installed) |
| Danger | `#f87171` | `#ffffff` | none | Destructive (Uninstall, Delete) |
| Secondary | `#22262f` | `#a0a8b4` | `#2e3340` | Secondary actions |
| Ghost | transparent | `#a0a8b4` | none | Tertiary, icon-only |
| Disabled | `#22262f` | `#4a4f5a` | none | Inactive state |

### Badge Variants

| Badge | Background | Text | Icon |
|---|---|---|---|
| Installed | `rgba(52,211,153,0.15)` | `#34d399` | ✓ check |
| Update | `rgba(74,144,217,0.15)` | `#4a90d9` | ⬆ arrow-up |
| New | `rgba(167,139,250,0.15)` | `#a78bfa` | ✨ sparkles |
| Error | `rgba(248,113,113,0.15)` | `#f87171` | ⚠ alert |
| Favorite | `rgba(244,114,182,0.15)` | `#f472b6` | ★ star |

### Input Fields

- Background: `#22262f`
- Border: `#2e3340` (1px)
- Focus border: `#4a90d9`
- Text: `#f0f2f5`
- Placeholder: `#6b7280`
- Height: 40px
- Padding: 12px horizontal
- Border-radius: 10px

---

## Key User Journey Flows

### Journey 1: First Launch → Browse Library
1. App opens → Loading spinner → Fetches config + metadata
2. Sidebar "Library" selected by default
3. Grid view loads with skeleton cards → real cards fade in
4. Amber banner: "No device connected. Download-only mode active."
5. User can search, filter, sort, browse immediately

### Journey 2: Connect Quest → Install Game
1. User plugs in Quest via USB
2. Title bar updates: `● Connected: Quest 3 (72% 🔋)` with green flash
3. Amber banner disappears
4. User clicks game card → Detail panel slides in
5. Clicks "Download & Install" → Button shows progress → Timeline modal
6. Complete → Green toast: "Beat Saber installed successfully"

### Journey 3: Queue Multiple Downloads
1. User clicks "Download" on multiple games from grid (context menu or button)
2. Status bar expands to show active download
3. Sidebar "Downloads" shows badge with queue count
4. Navigate to Downloads → see active + queued + completed
5. Drag to reorder, pause/cancel individual items

### Journey 4: Offline Mode
1. App launches without internet
2. Uses cached catalog and metadata
3. Info banner: "Offline mode — browsing cached catalog. Connect to internet for updates."
4. Can still install from already-downloaded local files
5. Download buttons show "Unavailable offline" state

### Journey 5: Multi-Device
1. Two devices connected (USB + WiFi)
2. Device selector appears in sidebar Device panel
3. User selects target device
4. All install/uninstall operations route to selected device
5. Title bar shows selected device info
