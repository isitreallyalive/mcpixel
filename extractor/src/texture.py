from PIL import Image

def apply_overlays(textures: dict[str, Image.Image]) -> dict[tuple[str, str | None], Image.Image]:
    """Apply overlays to a textures dict"""
    output = {}

    # find overlays
    overlays = {
        name: img
        for name, img in textures.items()
        if name.endswith("_stained_glass")
    }

    # find bases
    bases = {
        name: img
        for name, img in textures.items()
        if name not in overlays
    }

    for base_name, base_img in bases.items():
        # add the texture alone
        output[(base_name, None)] = base_img

        # add all possible overlays
        for overlay_name, overlay_img in overlays.items():
            # apply the overlay
            result = base_img.copy()
            overlay_img = overlay_img.resize(base_img.size, Image.Resampling.NEAREST)
            result.paste(overlay_img, mask=overlay_img.split()[3])

            # add it to the output
            output[(base_name, overlay_name)] = result

    return output

