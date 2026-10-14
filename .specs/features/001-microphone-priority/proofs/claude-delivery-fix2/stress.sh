#!/bin/bash
# usage: stress.sh <binary> <filter> <out_file> ; 8 workers x 20 rounds, cwd = src-tauri
BIN=$1; FILTER=$2; OUT=$3
cd /Users/julienm/projects/handy-wt/001-microphone-priority/src-tauri
: > $OUT.runs
for w in 1 2 3 4 5 6 7 8; do
  ( for r in $(seq 1 20); do
      "$BIN" "$FILTER" --test-threads=8 > $OUT.w$w.r$r.log 2>&1; code=$?
      echo "worker=$w round=$r exit=$code $(grep -o 'test result: .*' $OUT.w$w.r$r.log | head -1)" >> $OUT.runs
      if [ $code -eq 0 ]; then rm -f $OUT.w$w.r$r.log; fi
    done ) &
done
wait
total=$(wc -l < $OUT.runs); fails=$(grep -vc "exit=0 " $OUT.runs)
echo "binary=$BIN sha256=$(shasum -a 256 $BIN | cut -d' ' -f1) filter=$FILTER workers=8 rounds=20 runs=$total failures=$fails" > $OUT
sort -t= -k2n $OUT.runs >> $OUT
rm -f $OUT.runs
