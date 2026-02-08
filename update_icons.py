import os
import shutil
import sys
from pathlib import Path
from PIL import Image

# Define paths
BASE_DIR = Path(__file__).parent
SRC_DIR = BASE_DIR / "static"
DST_DIR = BASE_DIR / "src-tauri" / "icons"

# Source files
SRC_PNG = SRC_DIR / "favicon.png"
SRC_ICO = SRC_DIR / "favicon.ico"
SRC_SVG = SRC_DIR / "favicon.svg"

def main():
    # 1. Verify source directory and files
    if not SRC_DIR.exists():
        print(f"Error: Source directory '{SRC_DIR}' does not exist.")
        return

    # Create destination directory if it doesn't exist
    DST_DIR.mkdir(parents=True, exist_ok=True)

    # 2. Process ICO (Generate from high-res PNG for better quality)
    # The user provided a source ICO, but to "maintain quality" (ensure it's not blurry), 
    # it is safer to regenerate it from the high-res PNG if available.
    if SRC_PNG.exists():
        dst_ico = DST_DIR / "icon.ico"
        try:
            with Image.open(SRC_PNG) as img:
                print(f"Generating high-quality icon.ico from {SRC_PNG} using LANCZOS resampling...")
                
                # Ensure RGBA for high quality resize (avoids palette issues)
                if img.mode != 'RGBA':
                    img = img.convert('RGBA')

                # Windows ICO usually needs these sizes
                icon_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
                
                # Generate high-quality resized images for each required size
                resampled_images = []
                for size in icon_sizes:
                    resampled = img.resize(size, resample=Image.Resampling.LANCZOS)
                    resampled_images.append(resampled)

                # Save the ICO. We use the largest one as the primary image and append the others.
                # Actually, according to spec, order doesn't strictly matter but usually largest first or last is fine.
                # Pillow handles it.
                # We sort by size descending generally for safety, but here we just pass them all.
                # The first image acts as the base, others are appended.
                # Let's use the largest (256x256) as the base.
                primary_icon = resampled_images.pop() # 256x256
                primary_icon.save(dst_ico, format='ICO', append_images=resampled_images)
                
                print(f"Updated: {dst_ico} with sizes {[s.size for s in [primary_icon] + resampled_images]}")
        except Exception as e:
            print(f"Error generating ICO from PNG: {e}")
            # Fallback to copying if generation fails
            if SRC_ICO.exists():
                print("Falling back to copying source ICO...")
                shutil.copy(SRC_ICO, dst_ico)
    elif SRC_ICO.exists():
        # Fallback if no PNG
        dst_ico = DST_DIR / "icon.ico"
        shutil.copy(SRC_ICO, dst_ico)
        print(f"Copied source ICO (PNG not found): {dst_ico}")

    # 3. Process SVG
    if SRC_SVG.exists():
        dst_svg = DST_DIR / "icon.svg" # Standardize name if needed, or keep favicon.svg? "icon.svg" is safer.
        try:
            shutil.copy(SRC_SVG, dst_svg)
            print(f"Updated: {dst_svg}")
        except Exception as e:
            print(f"Error copying SVG: {e}")
    else:
        print(f"Warning: {SRC_SVG} not found.")

    # 4. Process PNGs
    if SRC_PNG.exists():
        try:
            with Image.open(SRC_PNG) as img:
                # Ensure we have a high-quality source
                print(f"Processing PNG using source: {SRC_PNG} ({img.size})")

                # Define target sizes
                # These cover standard Tauri/Windows/Linux requirements
                targets = {
                    "32x32.png": (32, 32),
                    "128x128.png": (128, 128),
                    "128x128@2x.png": (256, 256),
                    "icon.png": (512, 512), # Main icon, usually 512x512
                    "Square30x30Logo.png": (30, 30),
                    "Square44x44Logo.png": (44, 44),
                    "Square71x71Logo.png": (71, 71),
                    "Square89x89Logo.png": (89, 89),
                    "Square107x107Logo.png": (107, 107),
                    "Square142x142Logo.png": (142, 142),
                    "Square150x150Logo.png": (150, 150),
                    "Square284x284Logo.png": (284, 284),
                    "Square310x310Logo.png": (310, 310),
                    "StoreLogo.png": (50, 50),
                }

                for name, size in targets.items():
                    # High-quality resize
                    # If the source is smaller, this might upscale, but user asked to use this file.
                    resized = img.resize(size, resample=Image.Resampling.LANCZOS)
                    dst_path = DST_DIR / name
                    resized.save(dst_path, format="PNG")
                    print(f"Generated: {name} {size}")

                # 5. Try to generate icon.icns (macOS)
                # This requires raw RGBA data or specific handling. 
                # Pillow's ICNS support is decent for basic saving.
                try:
                    dst_icns = DST_DIR / "icon.icns"
                    # ICNS supports specific sizes. We can pass the image and let Pillow handle it if it supports it.
                    # Best practice with Pillow: provide a sequence of images (sizes) to appended_images if supported or just save.
                    # But save(format='ICNS') with one image might auto-generate sizes or fail.
                    # A more robust way:
                    if img.mode != 'RGBA':
                        img = img.convert('RGBA')
                    
                    # We can try to manually create the icon-set for icns if we had icnsutil, but with just Pillow:
                    img.save(dst_icns, format='ICNS', sizes=[
                        (16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)
                    ])
                    print(f"Updated: {dst_icns}")
                except Exception as e:
                    print(f"Warning: Could not correct generate icon.icns (macOS icon): {e}")

        except Exception as e:
            print(f"Error processing PNGs: {e}")
    else:
        print(f"Error: {SRC_PNG} not found.")

if __name__ == "__main__":
    main()
