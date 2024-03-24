#!/usr/bin/env python3

"""
This script generates various sizes of icons from a single input image and saves them in a specified output directory.
It also creates an ICNS file containing these icons for macOS applications and an ICO file for Windows applications.

Install:
    pip3 install pillow==10.2.0 icnsutil==1.1.0

Usage:
    ./iconize.py --icon icon.png --output icons
"""

import argparse
import os
import shutil
import logging

from PIL import Image
from icnsutil import IcnsFile, IcnsType

logging.basicConfig(level=logging.INFO)


def iconize(icon_path, output_dir):
    if not os.path.isfile(icon_path):
        logging.error(f"The specified icon path does not exist: {icon_path}")
        return

    os.makedirs(output_dir, exist_ok=True)
    icon_image = Image.open(icon_path)
    icns = IcnsFile()

    sizes = {
        '32x32': (32, 32),
        '128x128': (128, 128),
        '128x128@2x': (256, 256),
        'Square30x30Logo': (30, 30),
        'Square44x44Logo': (44, 44),
        'Square71x71Logo': (71, 71),
        'Square89x89Logo': (89, 89),
        'Square107x107Logo': (107, 107),
        'Square142x142Logo': (142, 142),
        'Square150x150Logo': (150, 150),
        'Square284x284Logo': (284, 284),
        'Square310x310Logo': (310, 310),
        'StoreLogo': (50, 50)
    }

    # List to store the paths of the generated icons
    icon_paths = []

    for name, size in sizes.items():
        resized_icon = icon_image.resize(size, Image.LANCZOS)
        resized_icon_path = os.path.join(output_dir, f'{name}.png')
        resized_icon.save(resized_icon_path)
        icon_paths.append(resized_icon_path)
        try:
            icns.add_media(file=resized_icon_path)
        except IcnsType.CanNotDetermine as e:
            logging.warning(f"Can't determine type for {name}.png: {e}")
            continue

    icns_path = f'{output_dir}/icon.icns'
    ico_path = os.path.join(output_dir, 'icon.ico')
    icon_image.save(ico_path, format='ICO', sizes=[(16, 16), (32, 32), (48, 48), (64, 64)])
    icns.write(icns_path)

    # Add ICNS and ICO paths
    icon_paths.extend([icns_path, ico_path])

    # Generate the icon block string
    icon_block = ',\n        '.join([f'"{path}"' for path in icon_paths])
    icon_block_string = f'"icon": [\n        {icon_block}\n      ]'

    logging.info(f"Icons generated successfully in {output_dir}")
    logging.info(f"Add the following 'icon' block to your 'tauri.config.json':\n{icon_block_string}")

    return icon_block_string  # Return the icon block string for further use if needed


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate various sizes of icons from a single image.")
    parser.add_argument("--icon", type=str, required=True, help="Path to the source icon image.")
    parser.add_argument("--output", type=str, required=True, help="Directory to save the generated icons.")
    args = parser.parse_args()

    iconize(args.icon, args.output)
