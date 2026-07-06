BASEDIR="$(dirname $(readlink -f "$0"))"
cp -af $BASEDIR/powercfg.sh /data/powercfg.sh
cp $BASEDIR/powercfg.json /data/powercfg.json
chmod 755 /data/powercfg.sh
cur_powermode="/data/adb/modules/SZE_FINALIZER/config/config.txt"
if [ ! -f "$cur_powermode" ]; then
	touch "$cur_powermode"
	echo "powersave" > "$cur_powermode"
fi
