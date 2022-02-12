#!/usr/bin/env python3
import gzip

dictFiles = {}

with gzip.open("./english-words.txt.gz") as f:
  for word in f:
    word = word.strip()
    n = len(word)

    if n not in dictFiles:
      dictFiles[n] = open(f"{n}.txt", "w")

    dictFiles[n].write(word.decode("utf-8") + "\n")
  
for f in dictFiles.values():
  f.close()
