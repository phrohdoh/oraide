#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
}

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

        # read the content, adding the MPLv2 header
        with open(p, 'r+') as fh:
            contents = fh.read()
            contents = [
                f"{comment_prefix} This Source Code Form is subject to the terms of the Mozilla Public",
                f"{comment_prefix} License, v. 2.0. If a copy of the MPL was not distributed with this",
                f"{comment_prefix} file, You can obtain one at http://mozilla.org/MPL/2.0/.",
                "", # an empty line
                contents,
            ]

            new_content_str = '\n'.join(contents)

        # overwrite the file
        with open(p, 'w') as fh:
            fh.write(new_content_str)