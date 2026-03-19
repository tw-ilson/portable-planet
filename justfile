game := "portable-planet"
eboot := "target/mipsel-sony-psp/debug/EBOOT.PBP"
prx := "target/mipsel-sony-psp/debug/" + game + ".prx"
psplink_src := "/tmp/psplinkusb_build/psplink"
hostfs_pid := "/tmp/usbhostfs_pc.pid"
hostfs_log := "/tmp/usbhostfs_pc.log"

help:
    @just --list

psplink-start:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f "{{hostfs_pid}}" ] && kill -0 "$(cat {{hostfs_pid}})" 2>/dev/null; then
        echo "usbhostfs_pc already running (pid $(cat {{hostfs_pid}}))"
        exit 0
    fi
    nohup usbhostfs_pc "{{justfile_directory()}}/target/mipsel-sony-psp/debug" \
        > {{hostfs_log}} 2>&1 &
    echo $! > {{hostfs_pid}}
    echo "usbhostfs_pc started (pid $!), log: {{hostfs_log}}"

psplink-stop:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f "{{hostfs_pid}}" ]; then
        echo "not running (no pid file)"
        exit 0
    fi
    pid=$(cat {{hostfs_pid}})
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid"
        echo "usbhostfs_pc stopped (pid $pid)"
    else
        echo "process $pid not running"
    fi
    rm -f {{hostfs_pid}}

run:
    #!/usr/bin/env bash
    set -euo pipefail
    pspsh -e reset
    cargo psp
    pspsh -e "./{{game}}.prx"

install-psplink psp_root:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -d "{{psp_root}}/PSP/GAME" ]; then
        echo "error: no PSP filesystem at '{{psp_root}}'" >&2
        exit 1
    fi
    dest="{{psp_root}}/PSP/GAME/PSPLINK"
    mkdir -p "$dest"
    cp {{psplink_src}}/* "$dest/"
    echo "PSPLink installed to $dest"

upload psp_root:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -d "{{psp_root}}/PSP/GAME" ]; then
        echo "error: no PSP filesystem at '{{psp_root}}'" >&2
        exit 1
    fi
    dest="{{psp_root}}/PSP/GAME/{{game}}"
    mkdir -p "$dest"
    cp {{eboot}} "$dest/EBOOT.PBP"
    echo "Uploaded to $dest/EBOOT.PBP"
