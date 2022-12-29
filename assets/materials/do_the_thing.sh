#! /bin/bash


mkdir -p max
rm -f max/* -r
cp zips/*.zip max
RES=1024
PCT=50
rm $RES -rf
mkdir $RES

(cd max;
for f in *.zip; do
	unzip "$f";
done;

chmod a+w *;
for f in *.zip; do
	mv "$f" /tmp;
done;
chmod a+w *

for f in *; do
	mkdir -p "../$RES/$f";
done

for f in */*.jpg */*.png; do
	echo convert -scale $PCT% "$f" "../$RES/$f"
	convert -scale $PCT% "$f" "../$RES/$f"
done

for f in */*.tif; do
	echo convert -scale $PCT% "$f" "../$RES/$(echo $f | sed s/tif/png/g)"
	convert -scale $PCT% "$f" "../$RES/$(echo $f | sed s/tif/png/g)"
done
)