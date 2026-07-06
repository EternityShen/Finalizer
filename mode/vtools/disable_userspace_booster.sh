    for f in /sys/module/migt/parameters/*; do
        chmod 0000 $f
    done

    stop vendor.perfservice
    stop miuibooster

    stop oneplus_brain_service 2>/dev/null

    stop perfd 2>/dev/null

    stop vendor.power-hal-1-0
    stop vendor.power-hal-1-1
    stop vendor.power-hal-1-2
    stop vendor.power-hal-1-3
    stop vendor.oplus.ormsHalService-aidl-default