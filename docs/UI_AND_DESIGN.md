# UI Design & Aesthetics

## Overview
Llama Desktop prioritizes a premium, modern, and highly interactive user experience. This document outlines the design principles, color palettes, and component standards used in the application.

## Design Vision
- **Aesthetic**: Dark-themed, glassmorphic, and clean.
- **Interactivity**: Smooth transitions, hover effects, and real-time feedback (micro-animations).
- **Branding**: Dynamic icons and logos that automatically adapt to the specific AI model being used.

## 1. Color System
The application uses a curated Dark Mode palette:
- **Background**: Deep charcoal/black (`#0a0a0a`).
- **Surface**: Subtle translucency with glassmorphism (`rgba(255,255,255,0.05)`).
- **Accents**: 
  - **Primary**: Vibrant Indigo/Violet for main actions.
  - **Success**: Emerald green for "Server Running" states.
  - **Warning**: Amber for low resources or potential issues.
  - **Danger**: Rose/Red for errors or critical alerts.

## 2. Iconography & Logos

### Lucide Icons
Used for general UI actions (settings, delete, refresh, etc.).
- Consistent stroke weight (2px).
- Sized relative to surrounding text (usually 16px to 20px).

### Model Branding (`ModelLogo.svelte`)
A specialized component that maps model names to their official brand logos:
1. **Primary Source**: `svgl-svelte` (for high-quality brand SVGs).
2. **Secondary Fallback**: `svelte-simple-icons` (broad coverage of industry logos).
3. **Generic Fallback**: A generic "Box" or "Bot" icon if no brand match is found.

*Supported Brands include: OpenAI, Meta (Llama), Mistral, Google (Gemma), Microsoft (Phi), Qwen, etc.*

## 3. Key Components

### Model Card (`ModelCard.svelte`)
- **Visuals**: Features a custom "border gap" design where the bottom border is interrupted by the model's status text (Loaded/Warning/Offload).
- **Metrics**: Real-time progress bars for memory offloading.
- **Micro-animations**: Subtle scale-up on hover and status ball pulsing.

### Model Usage Graph (`ModelUsageGraph.svelte`)
- **Grid Layout**: A 5x10 visual grid (50 squares) representing total resource utilization (CPU/GPU).
- **Active State**: Squares glow and change color intensity based on load percentage.
- **Mock State**: Displays a dim, static pattern when the server is idle.

### Chat Interface
- **Message Bubbles**: Transparent backgrounds with side-borders to differentiate User vs Assistant.
- **Markdown Support**: Full syntax highlighting and LaTeX support for technical conversations.
- **Avatars**: Brand logos are used as avatars for the assistant to provide immediate visual context.

## 4. Typography
- **Primary Font**: `Inter` or system-default Sans-serif.
- **Hierarchy**: Clear distinction between headings and body text using weight (600 for headings, 400 for body).
- **Monospace**: `JetBrains Mono` or `Fira Code` for code blocks.

---
*Last updated: 2026-02-03*
