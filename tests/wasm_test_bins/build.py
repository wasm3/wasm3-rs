import subprocess
import shutil
import os

old_wd = os.getcwd()
os.chdir(os.path.dirname(os.path.realpath(__file__)))
subprocess.run(["cargo", "build", "--target", "wasm32-unknown-unknown"]).check_returncode()
shutil.copyfile("target/wasm32-unknown-unknown/debug/wasm_test_bins.wasm", "wasm_test_bins.wasm")
os.chdir(old_wd)
