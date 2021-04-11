#!/bin/bash

PARTS=8

WIDTH=256
HEIGHT=256

if ((WIDTH % PARTS)); then
  echo "${WIDTH} % ${PARTS} != 0 ! Aborting..."
  return 1
fi

STEP_X=$((WIDTH / PARTS))

MIN_X=0

for (( i=1; i<=PARTS; i++))
do
  FOO=$((MIN_X + STEP_X))

#          num  min.x   min.y max.x max.y
  submit ./job.sh "$i" "$MIN_X" "0" "$FOO" "$HEIGHT"

  MIN_X=$FOO
done
