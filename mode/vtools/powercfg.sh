#!/system/bin/sh

exists() {
    [ -e "$1" ] && return 0 || return 1
}

switch_mode() {
    case $1 in
        "powersave" | "balance")
            echo "$1" > /data/adb/modules/SZE_FINALIZER/config/config.txt
            ;;
        "performance" | "fast")
            echo "$1" > /dev/fas_rs/mode
            ;;
        *)
            echo "Failed to apply unknown mode '$1'."
            ;;
    esac
}

switch_mod() {
    echo "$1" > /data/adb/modules/SZE_FINALIZER/config/config.txt
}

case $1 in
    "powersave" | "balance" | "performance" | "fast")
        if exists /dev/fas_rs/mode; then
            switch_mode $1
        else
            switch_mod $1
        fi
        ;;
    "init")
        /data/powercfg.sh $(cat /data/adb/modules/SZE_FINALIZER/config/config.txt)
        ;;
    *)
        echo "Failed to apply unknown action '$1'."
        ;;
esac
