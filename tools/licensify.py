#!/usr/bin/env python3

# This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
# 
# oraide is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License version 3
# as published by the Free Software Foundation.
# 
# oraide is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
# 
# You should have received a copy of the GNU Affero General Public License
# along with oraide.  If not, see <https://www.gnu.org/licenses/>.

import os
import pathlib

start_dir_path = '.' # relative to **where this script was invoked from**

blacklist_dirs = set([
    '.git',
    '.vscode',
    'target',
])

map_file_ext_to_comment_prefix = {
    '.rs': '//',
    '.bash': '#',
    '.py': '#',
    '.ts': '//',
}

# AGPLv3
license_lines = [
    "This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.",
    "",
    "oraide is free software: you can redistribute it and/or modify",
    "it under the terms of the GNU Affero General Public License version 3",
    "as published by the Free Software Foundation.",
    "",
    "oraide is distributed in the hope that it will be useful,",
    "but WITHOUT ANY WARRANTY; without even the implied warranty of",
    "MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the",
    "GNU Affero General Public License for more details.",
    "",
    "You should have received a copy of the GNU Affero General Public License",
    "along with oraide.  If not, see <https://www.gnu.org/licenses/>.",
]

for root, dirs, files in os.walk(start_dir_path, topdown=True):
    # modify `dirs` in-place to remove the excluded directories
    dirs[:] = [d for d in dirs if d not in blacklist_dirs]

    for f in files:
        p = os.path.join(root, f)
        ext = ''.join(pathlib.Path(f).suffixes)

        if not ext:
            continue

        comment_prefix = map_file_ext_to_comment_prefix.get(ext)

        if not comment_prefix:
            # print(f"[info] ignoring due to extension not being in `map_file_ext_to_comment_prefix`: {p}")
            continue

        new_content_str = None

        # read the content, adding the license header
        with open(p, 'r+') as fh:
            file_content = fh.read()

            lines = [f"{comment_prefix} {l}" for l in license_lines] + [
                "", # an empty line
                file_content,
            ]

            new_content_str = '\n'.join(lines)

        # overwrite the file
        with open(p, 'w') as fh:
            fh.write(new_content_str)