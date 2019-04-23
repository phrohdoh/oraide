#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

##### What / Why / How #####
#
# - What
#
# This script makes it simple to intercept, typically for viewing by a human,
# the stdin and stdout of a process, such as a language server binary.
#
#
# - Why
#
# Debugging language servers is, in my experience, not simple.
# You may actually be debugging a server, or may be using an existing server
# as reference while building your own, or may simply be doing research.
#
# Regardless, seeing *all* of the input and output of the server process
# can be helpful.
#
# Perhaps the client is sending a malformed header section but the server is
# unhelpful and just errors with a message like "invalid headers."  Of course
# we can do better as software engineers but that's what we get sometimes.
#
# `interceptor.bash` to the rescue!  You can now see the *exact* message the
# client sent and, hopefully, figure out *what* about the headers is invalid.
#
#
# - How
# 
# The program `tee` is used to make a copy of the input and output, both of
# which are written to files at paths that you can configure.
#
# These streams (input and output) are written as-is so, depending on your
# data, you may need to do processing on the log files before you can easily
# read the data.
#
# For most, if not all, Language Server Protocol implementors the data will be
# plaintext, so you won't have to do any processing before viewing, yay!
#
#####

### provided variables, don't modify this section ###
this_script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
### end provided variables

### configuration

# NOTE: This is specifically for https://doc.rust-lang.org/cargo/ projects
#       Feel free to remove this (and later references to it) if it isn't
#       useful for you
# must be one of `debug` or `release`
profile="debug"

# the name of the executable to intercept I/O for
exe_name="oraide-language-server"

# the absolute path of the executable to intercept I/O for
abs_exe_path="$(dirname "${this_script_dir}")" # go up one level into `oraide-language-server`
abs_exe_path="$(dirname "${abs_exe_path}")"    # go up one level into `components`
abs_exe_path="$(dirname "${abs_exe_path}")"    # go up one level into `oraide` (repo root)
abs_exe_path="${abs_exe_path}/target/${profile}/${exe_name}"

# the file path to write process input to
input_log_file_path="/tmp/${exe_name}.in.log"

# the file path to write process output to
output_log_file_path="/tmp/${exe_name}.out.log"

### end configuration

# Rust module paths, which is what `RUST_LOG` uses, have `_` not `-`
exe_log_id=${exe_name//-/_}

cat - \
| tee "${input_log_file_path}" \
| RUST_LOG=${exe_log_id}=trace "${abs_exe_path}" $@ \
| tee "${output_log_file_path}"