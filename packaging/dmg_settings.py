# dmgbuild settings for the drag-and-drop installer window.
# Used by scripts/package.sh:  dmgbuild -s packaging/dmg_settings.py -D app=... -D background=... -D icon=... NAME OUT.dmg
import os.path

app = defines.get("app", "dist/NothingX.app")  # noqa: F821 (provided by dmgbuild)
appname = os.path.basename(app)

format = "UDZO"
filesystem = "HFS+"
files = [app]
symlinks = {"Applications": "/Applications"}
icon = defines.get("icon")  # noqa: F821  volume icon
badge_icon = None

background = defines.get("background")  # noqa: F821  640x400 (+@2x) TIFF
window_rect = ((200, 140), (640, 428))
default_view = "icon-view"
show_status_bar = False
show_tab_view = False
show_toolbar = False
show_pathbar = False
show_sidebar = False
show_icon_preview = False
include_icon_view_settings = True
include_list_view_settings = False
arrange_by = None
grid_offset = (0, 0)
grid_spacing = 100
scroll_position = (0, 0)
label_pos = "bottom"
text_size = 13
icon_size = 112
icon_locations = {
    appname: (170, 190),
    "Applications": (470, 190),
}
