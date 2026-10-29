"""Observe macOS vnode pinning and chmod without global creation-mask changes."""

import errno
import json
import os
import platform
import socket
import stat
import tempfile
from pathlib import Path


def metadata(value):
    return {
        "device": value.st_dev,
        "inode": value.st_ino,
        "mode": oct(stat.S_IMODE(value.st_mode)),
        "file_type": oct(stat.S_IFMT(value.st_mode)),
        "uid": value.st_uid,
        "nlink": value.st_nlink,
    }


def failure(error):
    return {
        "errno": error.errno,
        "errno_name": errno.errorcode.get(error.errno),
        "message": error.strerror,
    }


def observe_case(directory, replacement):
    path = directory / "listener.sock"
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
        listener.bind(str(path))
        listener.listen(1)
        result = {"replacement": replacement, "initial_path": metadata(os.lstat(path))}
        flags = os.O_EVTONLY | os.O_NOFOLLOW | os.O_CLOEXEC
        try:
            pinned = os.open(path, flags)
        except OSError as error:
            result["open"] = {"flags": flags, "result": -1, "error": failure(error)}
            return result
        result["open"] = {"flags": flags, "result": "descriptor", "error": None}
        replacement_socket = None
        try:
            result["pinned_before"] = metadata(os.fstat(pinned))
            if replacement != "none":
                path.unlink()
                if replacement == "regular":
                    path.write_bytes(b"retain replacement")
                    path.chmod(0o640)
                elif replacement == "symlink":
                    target = directory / "target"
                    target.write_bytes(b"retain target")
                    target.chmod(0o640)
                    path.symlink_to(target)
                else:
                    replacement_socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                    replacement_socket.bind(str(path))
            result["path_before_chmod"] = metadata(os.lstat(path))
            if replacement == "symlink":
                result["target_before_chmod"] = metadata(os.stat(path))
            try:
                os.fchmod(pinned, 0o600)
                result["fchmod"] = {"result": 0, "error": None}
            except OSError as error:
                result["fchmod"] = {"result": -1, "error": failure(error)}
            result["pinned_after"] = metadata(os.fstat(pinned))
            result["path_after"] = metadata(os.lstat(path))
            if replacement == "symlink":
                result["target_after"] = metadata(os.stat(path))
            result["path_matches_pin"] = all(
                result["path_after"][key] == result["pinned_after"][key]
                for key in ("device", "inode", "uid")
            )
            result["replacement_unchanged"] = (
                result["path_before_chmod"] == result["path_after"]
                if replacement != "none"
                else None
            )
            result["retained_path"] = path.exists() or path.is_symlink()
            return result
        finally:
            os.close(pinned)
            if replacement_socket is not None:
                replacement_socket.close()


def main():
    results = []
    # Darwin sockaddr_un paths are bounded; use a short isolated temporary root.
    with tempfile.TemporaryDirectory(prefix="hpfd-", dir="/tmp") as temporary:
        for replacement in ("none", "regular", "symlink", "socket"):
            directory = Path(temporary) / replacement
            directory.mkdir(mode=0o700)
            results.append(observe_case(directory, replacement))
    print(json.dumps({
        "platform": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "scope": "Native macOS experiment only; no Linux or production claim",
        "cases": results,
    }, indent=2))


if __name__ == "__main__":
    main()
