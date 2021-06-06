#!/usr/bin/python

import subprocess
from shutil import copyfile
from os import getcwd, chdir
from os.path import normpath, basename, dirname
from sys import argv

old_wd = getcwd()
path = argv[1];
bin_name = basename(normpath(path))
chdir(path)
subprocess.run(["cargo", "build", "--release", "--target", "wasm32-unknown-unknown"]).check_returncode()
copyfile("target/wasm32-unknown-unknown/release/{}.wasm".format(bin_name), "{}.wasm".format(bin_name))
chdir(old_wd)
