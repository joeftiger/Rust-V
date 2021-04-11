#!/bin/bash

PARTS=8

WIDTH=2048
HEIGHT=2048

if ((WIDTH % PARTS)); then
  echo "${WIDTH} % ${PARTS} != 0 ! Aborting..."
  return 1
fi

STEP_X=$((WIDTH / PARTS))

MIN_X=0

for (( i=1; i<=PARTS; i++))
do
  FOO=$((MIN_X + STEP_X))

################# num  min.x   min.y max.x max.y
  sbatch ./job.sh "$i" "$MIN_X" "0" "$FOO" "$HEIGHT"
  sleep 5
  MIN_X=$FOO
done
